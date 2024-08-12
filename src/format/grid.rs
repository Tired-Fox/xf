use owo_colors::OwoColorize;
use terminal_size::{terminal_size, Width};

use crate::{filter::Filter, sort::SortStrategy, style::Colorizer, FileSystem};

use super::Formatter;

pub struct Grid<S, F>(FileSystem<S, F>);

impl<S, F> Grid<S, F> {
    pub fn new(file_system: FileSystem<S, F>) -> Self {
        Self(file_system)
    }
}

impl<S: SortStrategy, F: Filter> Formatter for Grid<S, F> {
    fn print(&mut self, colorizer: Colorizer) -> std::io::Result<()> {
        let (Width(width), _) = terminal_size().unwrap();
        let width = width as usize;

        let entries = self.0.entries()?;
        let mut min = entries.len();
        {
            let mut pos = 0;
            let mut cols = 0;
            for entry in entries.iter() {
                if entry.file_name().len() + 2 + pos > width || cols >= min {
                    min = cols;
                    cols = 0;
                    pos = entry.file_name().len() + 2;
                }

                cols += 1;
                pos += entry.file_name().len() + 2; 
            }
        }

        let widths = entries.chunks(min).fold(vec![0;min], |mut acc, val| {
            for i in 0..val.len() {
                if val[i].file_name().len() > acc[i] {
                    acc[i] = val[i].file_name().len();
                }
            }
            acc
        });

        println!("{}", entries.chunks(min).map(|vals| {
            vals.iter().enumerate().map(|(i, v)| {
                format!("{}{}",
                    v.file_name().style(colorizer.file(v)),
                    (0..widths[i]-v.file_name().len()).map(|_| ' ').collect::<String>()
                )
            }).collect::<Vec<_>>().join("  ")
        }).collect::<Vec<_>>().join("\n"));
        Ok(())
    }
}
