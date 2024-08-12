mod grid;
pub use grid::Grid;

use std::io::Write;

use crate::{filter::Filter, sort::SortStrategy, style::Colorizer, FileSystem};

pub trait Formatter {
    fn print(&mut self, colorizer: Colorizer) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct List<S, F>(FileSystem<S, F>);

impl<S, F> List<S, F> {
    pub fn new(file_system: FileSystem<S, F>) -> Self {
        Self(file_system)
    }
}

impl<S: SortStrategy, F: Filter> Formatter for List<S, F> {
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
