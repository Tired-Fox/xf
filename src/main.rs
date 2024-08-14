use clap::ArgAction;
use owo_colors::{colors::xterm::Gray, Style};
use xf::{format::Formatter, style::{Colorizer, GroupMatch}, Directory, FileSystem};

fn main() {
    let matches = clap::Command::new("xf")
        .bin_name("xf")
        .display_name("xf")
        .arg(clap::Arg::new("path")
            .default_value(".")
            .action(clap::ArgAction::Set)
        )
        .arg(clap::Arg::new("pretty")
            .long("pretty")
            .action(ArgAction::SetTrue)
        )
        .get_matches();

    let path = matches.get_one::<String>("path").cloned().unwrap_or(".".to_string());
    let file_system = FileSystem::from(path)
        .with_sorter(Directory::default());

    let colorizer = Colorizer::default()
        .group("DIR", [GroupMatch::Directory], Style::default().blue())
        .group("HIDDEN", [GroupMatch::Hidden, GroupMatch::starts_with(".")], Style::default().fg::<Gray>())
        .group("IMAGE", [GroupMatch::extensions(["jpg", "png", "gif", "webp", "avif", "ico"])], Style::default().magenta())
        .group("CONFIG", [GroupMatch::filenames(["Cargo.toml", "config.toml"])], Style::default().yellow().underline())
        .group("EXE", [GroupMatch::Executable, GroupMatch::extensions(["exe", "sh"])], Style::default().green());

    if matches.get_flag("pretty") {
        println!();
        xf::format::List::new(file_system)
            .print(colorizer).unwrap();
        println!();
    } else {
        xf::format::Grid::new(file_system)
            .print(colorizer).unwrap();
    }
}
