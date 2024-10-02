use owo_colors::{colors::xterm, OwoColorize};

use crate::{ignore::GitIgnore, style::Colorizer, Entry, FileSystem};

use super::Formatter;

pub struct Tree(FileSystem, bool);

impl Tree {
    pub fn new(file_system: FileSystem, long: bool) -> Self {
        Self(file_system, long)
    }

    pub fn print_all(
        &self,
        entries: &[Entry],
        ignore: Option<GitIgnore>,
        indent: String,
        colorizer: &Colorizer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in entries[..entries.len().saturating_sub(1)]
            .iter()
            .filter(|e| {
                ignore
                    .as_ref()
                    .map(|v| {
                        v.include(e.path().strip_prefix(&self.0.path).unwrap())
                    })
                    .unwrap_or(true)
            })
        {
            let permissions = if self.1 {
                format!(
                    "{} {} {} ",
                    colorizer.permissions(entry),
                    colorizer.file_size(entry),
                    colorizer.date_modified(entry),
                )
            } else {
                String::new()
            };

            if entry.path.is_dir() {
                println!("{permissions}{indent}├ {}", colorizer.file(entry));
                let rec = entry.entries(&self.0)?;
                let gitignore = match entry.path.join(".gitignore").exists() {
                    true => Some(GitIgnore::try_from(entry.path.join(".gitignore"))?),
                    false => None,
                }.or_else(|| ignore.clone());
                self.print_all(&rec, gitignore, format!("{indent}│ "), colorizer)?;
            } else {
                println!("{permissions}{indent}├ {}", colorizer.file(entry));
            }
        }

        if let Some(last) = entries.last() {
            let permissions = if self.1 {
                format!(
                    "{} {} {} ",
                    colorizer.permissions(last),
                    colorizer.file_size(last),
                    colorizer.date_modified(last),
                )
            } else {
                String::new()
            };

            if last.path.is_dir() {
                println!("{permissions}{indent}└ {}", colorizer.file(last));
                let rec = last.entries(&self.0)?;
                let gitignore = match last.path.join(".gitignore").exists() {
                    true => Some(GitIgnore::try_from(last.path.join(".gitignore"))?),
                    false => None,
                };
                self.print_all(&rec, gitignore, format!("{indent}  "), colorizer)?;
            } else {
                println!("{permissions}{indent}└ {}", colorizer.file(last));
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
            format!(
                "{} {} {} ",
                colorizer.permissions(&parent),
                colorizer.file_size(&parent),
                colorizer.date_modified(&parent),
            )
        } else {
            String::new()
        };

        let parent_name = self
            .0
            .path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        println!(
            "{permissions}{}{}",
            format!("{}/", parent_name).fg::<xterm::Rose>(),
            self.0
                .path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .fg::<xterm::Rose>()
        );

        let gitignore = match parent.path.join(".gitignore").exists() {
            true => Some(GitIgnore::try_from(parent.path.join(".gitignore"))?),
            false => None,
        };
        self.print_all(&entries, gitignore, String::new(), &colorizer)?;

        Ok(())
    }
}
