use clap::ArgAction;
use owo_colors::{colors::xterm::Gray, Style};
use xf::{filter::Binary, format::Formatter, style::{Colorizer, GroupMatch}, Directory, FileSystem};

fn main() {
    let matches = clap::Command::new("xf")
        .bin_name("xf")
        .display_name("xf")
        .disable_help_flag(true)
        .arg(clap::Arg::new("path")
            .default_value(".")
            .action(clap::ArgAction::Set)
        )
        .arg(clap::Arg::new("pretty")
            .long("pretty")
            .short('p')
            .action(ArgAction::SetTrue)
        )
        .arg(clap::Arg::new("hidden")
            .long("hidden")
            .short('h')
            .action(ArgAction::SetTrue)
        )
        .arg(clap::Arg::new("help")
            .long("help")
            .action(ArgAction::Help)
        )
        .get_matches();

    let path = matches.get_one::<String>("path").cloned().unwrap_or(".".to_string());
    let file_system = if matches.get_flag("hidden") {
        FileSystem::from(path)
            .with_sorter(Directory::default())
            .with_filter(Directory::default().or(()))
    } else {
        FileSystem::from(path)
            .with_sorter(Directory::default())
    };

    let colorizer = Colorizer::default()
        .group("DIR", [GroupMatch::Directory], Style::default().blue())
        .group("HIDDEN", [GroupMatch::Hidden, GroupMatch::starts_with(".")], Style::default().fg::<Gray>())
        .group("IMAGE", [GroupMatch::extensions(["jpg", "png", "gif", "webp", "avif", "ico"])], Style::default().magenta())
        .group("CONFIG", [GroupMatch::filenames(["Cargo.toml", "config.toml"])], Style::default().yellow().underline())
        .group("EXE", [GroupMatch::Executable, GroupMatch::extensions(["exe", "sh"])], Style::default().green());

    if matches.get_flag("pretty") {
        xf::format::List::new(file_system)
            .print(colorizer).unwrap();
        println!();
    } else {
        xf::format::Grid::new(file_system)
            .print(colorizer).unwrap();
    }
}
