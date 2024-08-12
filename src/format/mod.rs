mod grid;

use std::{io::Write, os::windows::fs::MetadataExt};

use chrono::{Duration, Local};
pub use grid::Grid;
use owo_colors::OwoColorize;

use crate::{filter::Filter, sort::SortStrategy, style::Colorizer, FileSystem};

pub trait Formatter {
    fn print(&mut self, colorizer: Colorizer) -> std::io::Result<()>;
}

#[inline]
pub fn humansize(value: u64) -> String {
    match value {
        0 => "-".to_string(),
        // Bytes
        1..1_024 => value.to_string(),
        // Kilobytes
        1_024..1_048_576 => format!("{:.2}K", value as f32 / 1_024.0),
        // Megabytes
        1_048_576..1_073_741_824 => format!("{:.2}M", value as f32 / 1_048_576.0),
        // Gigbytes
        1_073_741_824..1_099_511_627_776 => format!("{:.2}G", value as f32 / 1_099_511_627_776.0),
        // Terabytes
        1_099_511_627_776..1_125_899_906_842_624 => format!("{:.2}T", value as f32 / 1_099_511_627_776.0),
        // Petabytes
        _ => format!("{:.2}P", value as f32 / 1_125_899_906_842_624.0)
    }
}

pub struct List<S, F>(FileSystem<S, F>);

impl<S, F> List<S, F> {
    pub fn new(file_system: FileSystem<S, F>) -> Self {
        Self(file_system)
    }
}

impl<S: SortStrategy, F: Filter> Formatter for List<S, F> {
    fn print(&mut self, colorizer: Colorizer) -> std::io::Result<()> {
        let mut stdout = std::io::stdout();
        for entry in self.0.entries()? {
            let date = entry.metadata().modified().map(|m| {
                let date = chrono::DateTime::<Local>::from(m);
                if Local::now() - date > Duration::weeks(52) {
                    date.format("%e %b  %Y")
                } else {
                    date.format("%e %b %H:%M")
                }.to_string()
            }).unwrap_or("-".to_string());

            let human_size = humansize(entry.metadata().file_size());

            writeln!(stdout, "{} {}{} {}{}  {}",
                entry.permissions(),
                (0..7usize.saturating_sub(human_size.len())).map(|_| ' ').collect::<String>(),
                human_size.style(colorizer.file_size()),
                (0..12usize.saturating_sub(date.len())).map(|_| ' ').collect::<String>(),
                date.style(colorizer.date_modified()),
                entry.file_name().style(colorizer.file(&entry)),
            )?;
        }
        stdout.flush()?;
        Ok(())
    }
}
