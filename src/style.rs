use std::{ops::{Range, RangeTo}, os::windows::fs::MetadataExt};

use chrono::Datelike;
use hashbrown::{HashMap, HashSet};
use owo_colors::{colors::xterm::Gray, OwoColorize, Style};

use crate::{permission::AccessRights, Entry};

pub struct GroupStyle {
    matcher_map: HashMap<&'static str, usize>,
    matchers: Vec<GroupMatch>,
    style: Style
}

impl GroupStyle {
    pub fn add_matcher(&mut self, matcher: GroupMatch) {
        if let Some(index) = self.matcher_map.get(matcher.as_ref()) {
            match (&mut self.matchers[*index], matcher) {
                (GroupMatch::Filename(curr), GroupMatch::Filename(new)) => curr.extend(new),
                (GroupMatch::Extension(curr), GroupMatch::Extension(new)) => curr.extend(new),
                _ => unreachable!()
            }
        } else {
            self.matcher_map.insert(matcher.as_ref(), self.matchers.len());
            self.matchers.push(matcher);
        }
    }

    pub fn matches(&self, entry: &Entry) -> bool {
        for matcher in self.matchers.iter() {
            if matcher.matches(entry) {
                return true;
            }
        }
        false
    }

    pub fn style(&self) -> Style {
        self.style
    }
} 

#[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumIs)]
pub enum GroupMatch {
    Directory,
    Hidden,
    Executable,
    StartsWith(String),
    EndsWith(String),
    Filename(HashSet<String>),
    Extension(HashSet<String>),
}

impl GroupMatch {
    pub fn filenames<I: IntoIterator<Item=S>, S: AsRef<str>>(filenames: I) -> Self {
        Self::Filename(filenames.into_iter().map(|v| v.as_ref().to_string()).collect())
    }

    pub fn extensions<I: IntoIterator<Item=S>, S: AsRef<str>>(extensions: I) -> Self {
        Self::Extension(extensions.into_iter().map(|v| v.as_ref().to_ascii_lowercase()).collect())
    }

    pub fn starts_with<S: ToString>(pattern: S) -> Self {
        Self::StartsWith(pattern.to_string())
    }

    pub fn ends_with<S: ToString>(pattern: S) -> Self {
        Self::EndsWith(pattern.to_string())
    }

    pub fn as_ref(&self) -> &'static str {
        match self {
            Self::Filename(_) => "Filename",
            Self::Extension(_) => "Extension",
            Self::Directory => "Directory",
            Self::Hidden => "Hidden",
            Self::Executable => "Executable",
            Self::StartsWith(_) => "StartsWith",
            Self::EndsWith(_) => "EndsWith"
        }
    }

    pub fn matches(&self, entry: &Entry) -> bool {
        match self {
            Self::Filename(names) => names.contains(entry.file_name()),
            Self::Extension(exts) => exts.contains(&entry.extension().unwrap_or_default()),
            Self::Directory => entry.is_dir(),
            Self::StartsWith(sw) => entry.file_name().starts_with(sw),
            Self::EndsWith(ew) => entry.file_name().ends_with(ew),
            Self::Hidden => entry.is_hidden(),
            Self::Executable => entry.is_executable(),
        }
    }
}

#[derive(Default)]
pub struct Colorizer {
    groups: HashMap<String, usize>,
    group_styles: Vec<GroupStyle>
}

impl Colorizer {
    pub fn group<S: AsRef<str>, I: IntoIterator<Item=GroupMatch>>(mut self, name: S, matchers: I, style: Style) -> Self {
        self.groups.insert(name.as_ref().to_string(), self.group_styles.len());

        // compress and optimize matchers
        let mut m = HashMap::<&str, GroupMatch>::new();
        for matcher in matchers {
            if m.contains_key(matcher.as_ref()) {
                match (m.get_mut(matcher.as_ref()).unwrap(), matcher) {
                    (GroupMatch::Filename(curr), GroupMatch::Filename(new)) => curr.extend(new),
                    (GroupMatch::Extension(curr), GroupMatch::Extension(new)) => curr.extend(new),
                    _ => unreachable!()
                }
            } else {
                m.insert(matcher.as_ref(), matcher);
            }
        }

        let m = m.into_iter().collect::<Vec<_>>();
        self.group_styles.push(GroupStyle {
            matcher_map: m.iter().enumerate().map(|(i, (k, _))| (*k, i)).collect(),
            matchers: m.into_iter().map(|(_, v)| v).collect(),
            style
        });
        self
    }

    pub fn add<S: AsRef<str>>(mut self, name: S, matcher: GroupMatch) -> Self {
        if let Some(index) = self.groups.get(&name.as_ref().to_string()) {
            self.group_styles[*index].add_matcher(matcher);
        }
        self
    }
}

#[inline]
pub fn humansize(value: u64) -> String {
    match value {
        0 => "-".to_string(),
        // Bytes
        1..1_024 => value.to_string(),
        // Kilobytes
        1_024..1_048_576 => format!("{}K", value / 1_024),
        // Megabytes
        1_048_576..1_073_741_824 => format!("{}M", value / 1_048_576),
        // Gigbytes
        1_073_741_824..1_099_511_627_776 => format!("{}G", value / 1_099_511_627_776),
        // Terabytes
        1_099_511_627_776..1_125_899_906_842_624 => format!("{}T", value / 1_099_511_627_776),
        // Petabytes
        _ => format!("{}P", value / 1_125_899_906_842_624)
    }
}

pub trait Spacer {
    fn spacer(self) -> String;
}
impl Spacer for Range<usize> {
    fn spacer(self) -> String {
        self.map(|_| ' ').collect::<String>()
    }
}

/// Helper to create either a dash (`-`) or a char representing the flag
pub trait ModeChar {
    const DASH: char = '-';
    //const DASH: char = '─';

    fn mode_char(&self, mode: char) -> char;
    fn mode_char_color(&self, mode: char, style: Style) -> String;
}

impl ModeChar for bool {
    #[inline]
    fn mode_char(&self, mode: char) -> char {
        if *self {
            mode
        } else {
            Self::DASH
        }
    }

    fn mode_char_color(&self, mode: char, style: Style) -> String {
        if *self {
            mode.style(style).to_string()
        } else {
            Self::DASH.style(Style::default().dimmed()).to_string()
        }
    }
}

impl Colorizer {
    pub fn file(&self, entry: &Entry) -> String {
        let mut style = Style::default();
        for m in self.group_styles.iter() {
            if m.matches(entry) {
                style = m.style();
            }
        }
        
        entry.file_name().style(style).to_string()
    }

    pub fn file_size(&self, entry: &Entry) -> String {
        if entry.metadata().is_symlink() {
            format!("   {}", '^'.fg::<Gray>())
        } else {
            let hs = humansize(entry.metadata().file_size());
            format!("{}{}", (0..hs.len()-4).spacer(), hs.fg::<Gray>())
        }
    }

    pub fn date_modified(&self, entry: &Entry) -> String {
        let date = entry.metadata().modified().map(|m| {
            let date = chrono::DateTime::<chrono::Local>::from(m);
            if date.year() < chrono::Local::now().year() {
                date.format("%e %b  %Y")
            } else {
                date.format("%e %b %H:%M")
            }.to_string()
        }).unwrap_or("-".to_string());

        format!("{}{}", (0..12usize.saturating_sub(date.len())).spacer(), date.blue())
    }

    fn access_rights(&self, buffer: &mut String, rights: &AccessRights) {
        buffer.push_str(rights.readable().mode_char_color('r', Style::default().yellow()).to_string().as_str());
        buffer.push_str(rights.writable().mode_char_color('w', Style::default().red()).to_string().as_str());
        buffer.push_str(rights.executable().mode_char_color('x', Style::default().green()).to_string().as_str());
    }

    fn file_type(&self, entry: &Entry) -> String {
        if entry.is_dir() {
            'd'.blue().to_string()
        } else {
            '.'.bold().to_string()
        }
    }

    pub fn permissions(&self, entry: &Entry) -> String {
        let mut result = self.file_type(entry);
        self.access_rights(&mut result, &entry.permissions().user().permissions);
        self.access_rights(&mut result, &entry.permissions().group().permissions);
        self.access_rights(&mut result, &entry.permissions().everyone().permissions);
        result
    }
}
