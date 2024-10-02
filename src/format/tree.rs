use owo_colors::{colors::xterm, OwoColorize};

use crate::{style::Colorizer, Entry, FileSystem};

use super::Formatter;

pub struct Tree(FileSystem, bool);
impl Tree {
    pub fn new(file_system: FileSystem, long: bool) -> Self {
        Self(file_system, long)
    }

    pub fn print_all(&self, entries: &[Entry], indent: usize, bar: bool, colorizer: &Colorizer) -> Result<(), Box<dyn std::error::Error>> {
        let offset = (0..indent).map(|_| if bar { "│ " } else { "  " }).collect::<String>();


        for entry in &entries[..entries.len().saturating_sub(1)] {
            let permissions = if self.1 {
                format!("{} {} {} ",
                    colorizer.permissions(entry),
                    colorizer.file_size(entry),
                    colorizer.date_modified(entry),
                )
            } else {
                String::new()
            };

            if entry.path.is_dir() {
                println!("{permissions}{offset}├ {}", colorizer.file(entry));
                let rec = entry.entries(&self.0)?;
                self.print_all(&rec, indent + 1, true, colorizer)?;
            } else {
                println!("{permissions}{offset}├ {}", colorizer.file(entry));
            }
        }

        if let Some(last) = entries.last() {
            let offset = (0..(indent*2)).map(|_| ' ').collect::<String>();

            let permissions = if self.1 {
                format!("{} {} {} ",
                    colorizer.permissions(last),
                    colorizer.file_size(last),
                    colorizer.date_modified(last),
                )
            } else {
                String::new()
            };

            if last.path.is_dir() {
                println!("{permissions}{offset}└ {}", colorizer.file(last));
                let rec = last.entries(&self.0)?;
                self.print_all(&rec, indent + 1, false, colorizer)?;
            } else {
                println!("{permissions}{offset}└ {}", colorizer.file(last));
            }
        }

        Ok(())
    }
}

impl Formatter for Tree {
    fn print(&mut self, colorizer: Colorizer) -> Result<(), Box<dyn std::error::Error>> {
        let entries = self.0.entries()?;

        let parent = Entry::try_from(self.0.path.as_path())?;
        let permissions = if self.1 {
            format!("{} {} {} ",
                colorizer.permissions(&parent),
                colorizer.file_size(&parent),
                colorizer.date_modified(&parent),
            )
        } else {
            String::new()
        };

        let parent_name = self.0.path.parent().unwrap().file_name().unwrap().to_str().unwrap();
        println!("{permissions}{}{}",
            format!("{}/", parent_name).fg::<xterm::Rose>(),
            self.0.path.file_name()
                .unwrap().to_str().unwrap()
                .fg::<xterm::Rose>()
        );
        self.print_all(&entries, 0, true, &colorizer)?;

        Ok(())
    }
}
