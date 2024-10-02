use clap::{ArgAction, ArgGroup};
use owo_colors::{colors::xterm::Gray, Style};
use xf::{filter::{Binary, Match}, format::Formatter, sort::{DateTime, Natural, Reverse, Size}, style::{Colorizer, GroupMatch}, Directory, FileSystem};

fn main() {
    let matches = clap::Command::new("xf")
        .bin_name("xf")
        .display_name("xf")
        .disable_help_flag(true)
        .arg(clap::Arg::new("path")
            .default_value(".")
            .action(clap::ArgAction::Set)
        )
        .arg(clap::Arg::new("help")
            .long("help")
            .action(ArgAction::Help)
        )
        .arg(clap::Arg::new("recursive")
            .long("tree")
            .short('R')
            .action(ArgAction::SetTrue)
        )
        .arg(clap::Arg::new("long")
            .long("long")
            .short('l')
            .action(ArgAction::SetTrue)
        )
        .arg(clap::Arg::new("filter")
            .long("filter")
            .short('f')
            .action(ArgAction::Set)
        )
        .arg(clap::Arg::new("all")
            .long("all")
            .short('a')
            .action(ArgAction::SetTrue)
        )
        .arg(clap::Arg::new("last-modified")
            .long("last-modified")
            .short('t')
            .action(ArgAction::SetTrue)
        )
        .arg(clap::Arg::new("reverse")
            .long("reverse")
            .short('r')
            .action(ArgAction::SetTrue)
        )
        .arg(clap::Arg::new("by-size")
            .long("by-size")
            .short('S')
            .action(ArgAction::SetTrue)
        )
        .group(ArgGroup::new("sorting")
            .args(["last-modified", "reverse", "by-size"])
            .multiple(false)
            .required(false)
        )
        .get_matches();

    let path = matches.get_one::<String>("path").cloned().unwrap_or(".".to_string());
    let mut file_system = FileSystem::from(path)
        .with_sorter(Directory::default());

    if matches.get_flag("all") {
        if let Some(f) = matches.get_one::<String>("filter") {
            file_system.set_filter(Directory::default().or(()).and(Match::new(f).unwrap()))
        } else {
            file_system.set_filter(Directory::default().or(()))
        }
    } else if let Some(f) = matches.get_one::<String>("filter") {
        file_system.set_filter(Match::new(f).unwrap())
    }

    if matches.get_flag("last-modified") {
        file_system.set_sorter(DateTime(Directory::default()));
    }

    if matches.get_flag("reverse") {
        file_system.set_sorter(Reverse(Directory(Reverse(Natural))));
    }

    if matches.get_flag("by-size") {
        file_system.set_sorter(Size(Directory::default()));
    }

    let colorizer = Colorizer::default()
        .group("DIR", [GroupMatch::Directory], Style::default().blue())
        .group("HIDDEN", [GroupMatch::Hidden, GroupMatch::starts_with("."), GroupMatch::extensions(["lock"])], Style::default().fg::<Gray>())
        .group("IMAGE", [GroupMatch::extensions(["jpg", "png", "gif", "webp", "avif", "ico"])], Style::default().magenta())
        .group("CONFIG", [GroupMatch::filenames(["Cargo.toml", "config.toml"])], Style::default().yellow().underline())
        .group("EXE", [GroupMatch::Executable, GroupMatch::extensions(["exe", "sh"])], Style::default().green());

    if matches.get_flag("recursive") {
        xf::format::Tree::new(file_system, matches.get_flag("long"))
            .print(colorizer).unwrap();
    } else if matches.get_flag("long") {
        xf::format::List::new(file_system)
            .print(colorizer).unwrap();
    } else {
        xf::format::Grid::new(file_system)
            .print(colorizer).unwrap();
    }
}
