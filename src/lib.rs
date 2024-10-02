pub mod format;
pub mod style;
pub mod sort;
pub mod filter;
pub mod permission;

use std::{cmp::Ordering, fs::{self, DirEntry, Metadata}, io, path::{Path, PathBuf}, rc::Rc};

use filter::{Filter, Not};
use permission::Perms;
use sort::{Natural, SortStrategy};

/// Wrapper around [`std::fs::DirEntry`]
///
/// Predetermines if it is a file or a directory along with providing helpers
/// around manipulating the entries.
#[derive(Debug, Clone)]
pub struct Entry {
    entry_type: EntryType,
    permissions: Perms,
    meta: Metadata,
    path: PathBuf,
}

#[derive(Debug, PartialEq, Clone, Copy, strum_macros::EnumIs)]
pub enum EntryType {
    File,
    Dir,
}

impl Entry {
    pub fn etype(&self) -> EntryType {
        self.entry_type
    }

    pub fn permissions(&self) -> &Perms {
        &self.permissions
    }

    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn file_name(&self) -> &str {
        self.path().file_name().and_then(|v| v.to_str()).unwrap_or("")
    }

    pub fn extension(&self) -> Option<String> {
        self.path()
            .extension()
            .and_then(|v| v.to_str().map(ToString::to_string))
    }

    pub fn is_dir(&self) -> bool {
        self.entry_type == EntryType::Dir
    }

    pub fn is_file(&self) -> bool {
        self.entry_type == EntryType::File
    }

    pub fn is_hidden(&self) -> bool {
        self.is_dot() || self.permissions().is_hidden()
    }

    pub(crate) fn is_dot(&self) -> bool {
        self.file_name().starts_with(".")
    }

    pub fn is_executable(&self) -> bool {
        self.permissions().user().executable()
    }
}

impl Entry {
    pub fn entries(&self, parent: &FileSystem) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
        if !self.is_dir() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Entry is not a directory").into());
        }

        let mut entries = fs::read_dir(&self.path)?
            .filter_map(|v| match v {
                Ok(v) => {
                    // PERF: Handle error
                    let entry = Entry::try_from(v).ok()?;
                    parent.filters.keep(&entry).then_some(entry)
                },
                _ => None
            })
            .collect::<Vec<_>>();

        entries.sort_by(|f, s| parent.sorter.compare(f, s));

        Ok(entries)
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        match (self.entry_type, other.entry_type) {
            (EntryType::File, EntryType::File) => self.path() == other.path(),
            (EntryType::Dir, EntryType::Dir) => self.path() == other.path(),
            _ => false
        }
    }
}
impl Eq for Entry {}

impl TryFrom<DirEntry> for Entry {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let entry_type = if value.path().is_dir() {
            EntryType::Dir
        } else {
            EntryType::File
        };

        Ok(Self {
            entry_type,
            permissions: Perms::try_from(&value)?,
            //permissions: Perms::default(),
            meta: value.metadata().unwrap(),
            path: value.path().to_path_buf(),
        })
    }
}

/// Helper to normalize `~` and other path features along with canonicalize the path
trait NormalizeCanonicalize {
    fn normalize_and_canonicalize(&self) -> Result<PathBuf, std::io::Error>;
}

impl<A: AsRef<str>> NormalizeCanonicalize for A {
    fn normalize_and_canonicalize(&self) -> Result<PathBuf, std::io::Error> {
        let mut path = self.as_ref().to_string();
        if path.starts_with('~') {
            path.replace_range(..1, dirs::home_dir().unwrap().display().to_string().as_str());
        }
        dunce::canonicalize(path)
    }
}

/// Main logic for transforming, sorting, and filtering file entries
pub struct FileSystem {
    path: PathBuf,
    filters: Rc<dyn Filter>,
    sorter: Rc<dyn SortStrategy>,
}

impl std::fmt::Debug for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XF")
            .field("path", &self.path)
            .finish()
    } 
}

impl Clone for FileSystem {
    fn clone(&self) -> Self {
        FileSystem {
            path: self.path.clone(),
            filters: self.filters.clone(),
            sorter: self.sorter.clone(),
        }
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        let path = std::env::current_dir().unwrap().display().to_string();
        Self {
            path: path.normalize_and_canonicalize().expect("Could not find the path specified"),
            filters: Rc::new(Not::<Hidden>::default()),
            sorter: Rc::new(()),
        }
    }
}

impl FileSystem {
    pub fn new<P: AsRef<Path>, S: SortStrategy + 'static, F: Filter + 'static>(path: P, sorter: S, filters: F) -> FileSystem {
        let path = path.as_ref().display().to_string();
        FileSystem {
            path:  path.normalize_and_canonicalize().expect("Could not find the path specified"),
            filters: Rc::new(filters),
            sorter: Rc::new(sorter),
        }
    }
}

impl FileSystem {
    pub fn with_sorter<S: SortStrategy + 'static>(self, sorter: S) -> FileSystem {
        FileSystem {
            path: self.path,
            filters: self.filters,
            sorter: Rc::new(sorter),
        }
    }

    pub fn with_filter<F: Filter + 'static>(self, filters: F) -> FileSystem {
        FileSystem {
            path: self.path,
            filters: Rc::new(filters),
            sorter: self.sorter,
        }
    }
}

impl<P: AsRef<Path>> From<P> for FileSystem {
    fn from(value: P) -> Self {
        let value = value.as_ref().display().to_string();
        FileSystem {
            path:  value.normalize_and_canonicalize().expect("Could not find the path specified"),
            filters: Rc::new(Not::<Hidden>::default()),
            sorter: Rc::new(()),
        }
    }
}

impl FileSystem {
    pub fn entries(&self) -> Result<Vec<Entry>, Box<dyn std::error::Error>> {
        let mut entries = fs::read_dir(&self.path)?
            .filter_map(|v| match v {
                Ok(v) => {
                    // PERF: Handle error
                    let entry = Entry::try_from(v).ok()?;
                    self.filters.keep(&entry).then_some(entry)
                },
                _ => None
            })
            .collect::<Vec<_>>();

        entries.sort_by(|f, s| self.sorter.compare(f, s));

        Ok(entries)
    }
}


/// A sorter that will sort directories first
pub struct Directory<T = Natural>(pub T);
impl Default for Directory {
    fn default() -> Self {
        Self(Natural)
    }
}
impl<T: Clone> Clone for Directory<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())        
    }
}

impl<T: SortStrategy> SortStrategy for Directory<T> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        match (first.entry_type, second.entry_type) {
            (EntryType::Dir, EntryType::File) => Ordering::Less,
            (EntryType::File, EntryType::Dir) => Ordering::Greater,
            _ => {
                self.0.compare(first, second)
            }
        }
    }
}
impl Filter for Directory {
    fn keep(&self, entry: &Entry) -> bool {
        entry.is_dir()
    }
}

/// A sorter that will sort hidden files first
pub struct Hidden<T = Natural>(T);
impl<T: Default> Default for Hidden<T> {
    fn default() -> Self {
        Self(T::default())
    }
}
impl<T: Clone> Clone for Hidden<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: PartialEq> PartialEq for Hidden<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T: SortStrategy> SortStrategy for Hidden<T> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        match (first.is_hidden(), second.is_hidden()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => self.0.compare(first, second)
        }
    }
}
impl Filter for Hidden {
    fn keep(&self, entry: &Entry) -> bool {
        entry.is_hidden()
    }
}
