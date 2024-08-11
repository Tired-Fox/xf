use std::fmt::Display;

use terminal_size::{terminal_size, Width};

use crate::{filter::Filter, sort::SortStrategy, EntryType, FileSystem};

use super::Formatter;

pub struct Grid<S, F> {
    pub file_system: FileSystem<S, F>,
    width: usize,
    column: usize,
    // TODO: Store colors and other display options.
}

impl<S, F> Grid<S, F> {
    pub fn new(file_system: FileSystem<S, F>) -> Self {
        let (Width(width), _) = terminal_size().unwrap();
        Self {
            file_system,
            width: width as usize,
            column: 0,
        }
    }
}

impl<S: SortStrategy, F: Filter> Formatter for Grid<S, F> {
    fn write_dir(&self, dir: &crate::Entry) {
        let name = dir.file_name();
        if name.len() + 2 >= self.width {
            println!();
        }

        print!("\x1b[34m{}/\x1b[39m  ", name)
    }

    fn write_file(&self, file: &crate::Entry) {
        let name = file.file_name();
        if name.len() + 2 >= self.width {
            println!();
        }

        if file.is_dot() || file.is_hidden() {
            print!("\x1b[38;5;8m{}\x1b[39m  ", name)
        } else {
            print!("{}  ", name)
        }
    }

    fn print(&self) -> std::io::Result<()> {
        for entry in self.file_system.iter()? {
            match entry.entry_type {
                EntryType::Dir => self.write_dir(&entry),
                EntryType::File => self.write_file(&entry),
            }
        }
        Ok(())
    }
}
