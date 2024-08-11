use std::path::Path;

use owo_colors::{colors::xterm::Gray, OwoColorize, Style};
use xf::{format::Formatter, permission::{ModeChar, Perms}, FileSystem};

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
            let extension = self.extension().map(|v| v.to_str().unwrap_or(""));
            if let Some(extension) = extension {
                // TODO: Change this to some sort of map that resolves the extension to a styling
                let style = match extension.to_lowercase().as_str() {
                    "png" | "gif" | "jpg" | "jpeg" => Style::default().purple(),
                    _ => Style::default()
                };
                name.style(style).to_string()
            } else {
                name.to_string()
            }
        }
    }
}


fn main() {
    let matches = clap::Command::new("xf")
        .bin_name("xf")
        .display_name("xf")
        .arg(clap::Arg::new("path")
            .default_value(".")
            .action(clap::ArgAction::Set)
        )
        .get_matches();

    let path = matches.get_one::<String>("path").cloned().unwrap_or(".".to_string());
    let file_system = FileSystem::from(path)
        .with_filter(());

    xf::format::Grid::new(file_system)
        .print().unwrap();
}
