mod grid;

pub use grid::Grid;

use crate::Entry;

pub trait Formatter {
    fn write_dir(&self, dir: &Entry);
    fn write_file(&self, file: &Entry);
    fn print(&self) -> std::io::Result<()>;
}
