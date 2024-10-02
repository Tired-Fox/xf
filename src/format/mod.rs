mod grid;
pub use grid::Grid;

use std::io::Write;

use crate::{style::Colorizer, FileSystem};

pub trait Formatter {
    fn print(&mut self, colorizer: Colorizer) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct List(FileSystem);

impl List {
    pub fn new(file_system: FileSystem) -> Self {
        Self(file_system)
    }
}

impl Formatter for List {
    fn print(&mut self, colorizer: Colorizer) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdout = std::io::stdout();
        for entry in self.0.entries()? {
            writeln!(stdout, "{} {} {}  {}",
                colorizer.permissions(&entry),
                colorizer.file_size(&entry),
                colorizer.date_modified(&entry),
                colorizer.file(&entry),
            )?;
        }
        stdout.flush()?;
        Ok(())
    }
}
