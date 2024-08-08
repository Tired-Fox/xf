use std::{cmp::Ordering, fs::{self, DirEntry}, path::Path};

use owo_colors::{colors::xterm::Gray, OwoColorize, Style};
use windows::{core::PCSTR, Win32::Storage::FileSystem::GetFileAttributesA};

#[derive(Debug)]
enum Entry {
    File(DirEntry),
    Dir(DirEntry),
}

pub trait IsHidden {
    fn is_hidden(&self) -> bool;
}

impl IsHidden for DirEntry {
    fn is_hidden(&self) -> bool {
        self.path().file_name().and_then(|v| v.to_str().map(|v| v.starts_with("."))).unwrap_or_default()
    }
}

pub trait ModeChar {
    fn mode_char(&self, mode: char) -> char;
    fn mode_char_color(&self, mode: char, style: Style) -> String;
}

impl ModeChar for bool {
    #[inline]
    fn mode_char(&self, mode: char) -> char {
        if *self { mode } else { '─' }
    } 

    fn mode_char_color(&self, mode: char, style: Style) -> String 
    {
        if *self {
            mode.style(style).to_string()
        } else {
            '─'.style(Style::default().fg::<Gray>().dimmed()).to_string()
        }
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

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Dir(_), Self::File(_)) => Ordering::Less,
            (Self::File(_), Self::Dir(_)) => Ordering::Greater,
            (Self::File(s) | Self::Dir(s), Self::File(o) | Self::Dir(o)) => {
                if s.path() == o.path() {
                    Ordering::Equal
                } else if !s.is_hidden() && o.is_hidden() {
                    Ordering::Greater
                } else if s.is_hidden() && !o.is_hidden() {
                    Ordering::Less
                } else {
                    s.path().cmp(&o.path())
                }
            }
        }
    }
}

impl From<DirEntry> for Entry {
    fn from(value: DirEntry) -> Self {
        if value.path().is_dir() {
            Self::Dir(value)
        } else {
            Self::File(value)
        }
    }
}

pub trait Colorize {
    fn colorize(&self) -> String;
}

impl Colorize for Perms {
    fn colorize(&self) -> String {
        match self {
            Self::Unix(_) => String::new(),
            Self::Windows { directory, archive, readonly, hidden, system, reparse_point } => {
                format!("{}{}{}{}{}{}",
                    directory.mode_char_color('d', Style::default().blue()),
                    archive.mode_char_color('a', Style::default().purple()),
                    readonly.mode_char_color('r', Style::default().yellow()),
                    hidden.mode_char_color('h', Style::default().fg::<Gray>()),
                    system.mode_char_color('s', Style::default().red()),
                    reparse_point.mode_char_color('l', Style::default().cyan())
                )
            }
        }
    }
}

impl Colorize for Path {
    fn colorize(&self) -> String {
        let name = self.file_name().unwrap().to_string_lossy();
        if self.is_dir() {
            name.blue().to_string()
        } else {
            name.to_string()
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RWE {
    read: bool,
    write: bool,
    execute: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Perms {
    Unix(u32),
    Windows {
        directory: bool,
        archive: bool,
        readonly: bool,
        hidden: bool,
        system: bool,
        reparse_point: bool
    },
}

impl Perms {
    pub fn from_windows(value: u32) -> Self {
        Self::Windows {
            directory: value & 0x0010 != 0,
            archive: value & 0x0020 != 0,
            readonly: value & 0x0001 != 0,
            hidden: value & 0x0002 != 0,
            system: value & 0x0004 != 0,
            reparse_point: value & 0x0400 != 0,
        }
    }
}

impl std::fmt::Display for Perms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unix(_) => Ok(()),
            Self::Windows { directory, archive, readonly, hidden, system, reparse_point } => {
                write!(f, "{}{}{}{}{}{}",
                    directory.mode_char('d'),
                    archive.mode_char('a'),
                    readonly.mode_char('r'),
                    hidden.mode_char('h'),
                    system.mode_char('s'),
                    reparse_point.mode_char('l')
                )
            }
        }
    }
}

impl From<&DirEntry> for Perms {
    fn from(value: &DirEntry) -> Self {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = value.metadata().unwrap().permissions();
            let _ = permissions.mode();
            Perms::Unix(0)
        }

        #[cfg(target_os = "windows")]
        unsafe {
            let mut path = value.path().display().to_string().replace('/', "\\");
            path.push('\0');
            let attributes = GetFileAttributesA(PCSTR::from_raw(path.as_ptr()));
            Perms::from_windows(attributes)
        }
    }
}

fn main() {
    let mut matches = clap::Command::new("xf")
        .bin_name("xf")
        .display_name("xf")
        .arg(clap::Arg::new("path")
            .default_value(".")
            .action(clap::ArgAction::Set)
        )
        .get_matches();

    let mut path = matches.get_one::<String>("path").cloned().unwrap_or(std::env::current_dir().unwrap().display().to_string());
    if path.starts_with("~") {
        path.replace_range(..1, dirs::home_dir().unwrap().display().to_string().as_str());
    }
    let path = dunce::canonicalize(path).expect("Could not find the path specified");
    println!("xf {path:?}");

    let mut entries = fs::read_dir(&path)
        .expect("Couldn't read directory")
        .filter_map(|v| v.ok().map(Entry::from))
        .collect::<Vec<_>>();

    entries.sort();

    println!("{}     {}", "Mode".underline(), "Name".underline());
    for entry in entries {
        match entry {
            Entry::File(entry) => {
                println!("{}   {}", Perms::from(&entry).colorize(), entry.path().colorize());
            },
            Entry::Dir(entry) => {
                println!("{}   {}", Perms::from(&entry).colorize(), entry.path().colorize());
            }
        }
    }
}
