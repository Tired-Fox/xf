pub mod sort;
pub mod filter;
pub mod permission;

use std::{ffi::OsString, fs::{self, DirEntry}, io, path::{Path, PathBuf}};

use filter::Filter;
use sort::{Natural, SortStrategy};

/// Wrapper around [`std::fs::DirEntry`]
///
/// Predetermines if it is a file or a directory along with providing helpers
/// around manipulating the entries.
#[derive(Debug, strum_macros::EnumIs, strum_macros::AsRefStr)]
pub enum Entry {
    File(DirEntry),
    Dir(DirEntry),
}

impl Entry {
    pub fn as_entry(&self) -> &DirEntry {
        match self {
            Self::File(entry) => entry,
            Self::Dir(entry) => entry
        }
    }

    pub fn file_name(&self) -> OsString {
        self.as_entry().file_name()
    }

    pub fn extension(&self) -> Option<String> {
        self.as_entry()
            .path()
            .extension()
            .and_then(|v| v.to_str().map(ToString::to_string))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::File(s), Self::File(o)) => s.path() == o.path(),
            (Self::Dir(s), Self::Dir(o)) => s.path() == o.path(),
            _ => false
        }
    }
}
impl Eq for Entry {}

impl From<DirEntry> for Entry {
    fn from(value: DirEntry) -> Self {
        if value.path().is_dir() {
            Self::Dir(value)
        } else {
            Self::File(value)
        }
    }
}

/// Helper to check if the path is hidden
pub(crate) trait IsHidden {
    fn is_hidden(&self) -> bool;
}

impl IsHidden for DirEntry {
    fn is_hidden(&self) -> bool {
        self.path().file_name().and_then(|v| v.to_str().map(|v| v.starts_with("."))).unwrap_or_default()
    }
}

/// Helper to normalize `~` and other path features along with canonicalize the path
trait NormalizeCanonicalize {
    fn normalize_and_canonicalize(&self) -> Result<PathBuf, std::io::Error>;
}

impl<A: AsRef<str>> NormalizeCanonicalize for A {
    fn normalize_and_canonicalize(&self) -> Result<PathBuf, std::io::Error> {
        let mut path = self.as_ref().to_string();
        if path.starts_with("~") {
            path.replace_range(..1, dirs::home_dir().unwrap().display().to_string().as_str());
        }
        dunce::canonicalize(path)
    }
}

/// Main logic for transforming, sorting, and filtering file entries
pub struct XF {
    path: PathBuf,
    sorters: Box<dyn SortStrategy>,
    filters: Box<dyn Filter>,
}

impl std::fmt::Debug for XF {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XF")
            .field("path", &self.path)
            .finish()
    } 
}

impl Default for XF {
    fn default() -> Self {
        let path = std::env::current_dir().unwrap().display().to_string();
        Self {
            path: path.normalize_and_canonicalize().expect("Could not find the path specified"),
            sorters: Box::new(Natural),
            filters: Box::new(())
        }
    }
}

impl XF {
    pub fn new<P: AsRef<Path>, T: SortStrategy + 'static, F: Filter + 'static>(path: P, sorters: T, filters: F) -> Self {
        let path = path.as_ref().display().to_string();
        Self {
            path:  path.normalize_and_canonicalize().expect("Could not find the path specified"),
            sorters: Box::new(sorters),
            filters: Box::new(filters),
        }
    }

    pub fn iter(&self) -> io::Result<XFIter> {
        let mut entries = fs::read_dir(&self.path)?
            .filter_map(|v| match v {
                Ok(v) => {
                    let entry = Entry::from(v);
                    self.filters.keep(&entry).then_some(entry)
                },
                _ => None
            })
            .collect::<Vec<_>>();

        entries.sort_by(|f, s| self.sorters.compare(f, s));

        Ok(XFIter(entries))
    }
}

/// Iterator over the entires in the filesystem. This will apply filters and sorting
/// provided by the [`XF`] object.
pub struct XFIter(Vec<Entry>);
impl Iterator for XFIter {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

