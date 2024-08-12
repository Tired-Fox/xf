#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;
use std::fs::DirEntry;

use owo_colors::{OwoColorize, Style};

use windows::core::HRESULT;
#[cfg(target_os = "windows")]
use windows::{core::PCSTR, Win32::Storage::FileSystem::{GetFileAttributesA, GetBinaryTypeA}};


/// Helper to create either a dash (`-`) or a char representing the flag
pub trait ModeChar {
    const DASH: char = '-';
    //const DASH: char = '─';

    fn mode_char(&self, mode: char) -> char;
    fn mode_char_color(&self, mode: char, style: Style) -> String;
}

impl ModeChar for bool {
    #[inline]
    fn mode_char(&self, mode: char) -> char {
        if *self { mode } else { Self::DASH }
    } 

    fn mode_char_color(&self, mode: char, style: Style) -> String 
    {
        if *self {
            mode.style(style).to_string()
        } else {
            Self::DASH.style(Style::default().dimmed()).to_string()
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
        executable: bool,
        directory: bool,
        archive: bool,
        readonly: bool,
        hidden: bool,
        system: bool,
        reparse_point: bool
    },
}

impl Perms {
    pub fn from_windows(value: u32, executable: bool) -> Self {
        Self::Windows {
            executable,
            directory: value & 0x0010 != 0,
            archive: value & 0x0020 != 0,
            readonly: value & 0x0001 != 0,
            hidden: value & 0x0002 != 0,
            system: value & 0x0004 != 0,
            reparse_point: value & 0x0400 != 0,
        }
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            Self::Windows { hidden, .. } => *hidden,
            // TODO: Check based on unix mode filetype
            _ => false
        }
    }
}

impl std::fmt::Display for Perms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unix(_) => unimplemented!(),
            Self::Windows { executable, directory, archive, readonly, hidden, system, reparse_point } => {
                write!(f, "{}{}{}{}{}{}{}",
                    directory.mode_char('d'),
                    archive.mode_char('a'),
                    readonly.mode_char('r'),
                    hidden.mode_char('h'),
                    system.mode_char('s'),
                    reparse_point.mode_char('l'),
                    executable.mode_char('x')
                )
            }
        }
    }
}

impl From<&DirEntry> for Perms {
    fn from(value: &DirEntry) -> Self {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let permissions = value.metadata().unwrap().permissions();
            let _ = permissions.mode();
            unimplemented!()
        }

        #[cfg(target_os = "windows")]
        unsafe {
            let mut path = value.path().display().to_string().replace('/', "\\");
            path.push('\0');
            let mut binary_type = 0u32;
            Perms::from_windows(
                GetFileAttributesA(PCSTR::from_raw(path.as_ptr())),
                GetBinaryTypeA(PCSTR::from_raw(path.as_ptr()), &mut binary_type as *mut _).is_ok()
            )
        }
    }
}
