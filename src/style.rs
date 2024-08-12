use hashbrown::{HashMap, HashSet};
use owo_colors::{colors::xterm::Gray, OwoColorize, Style};

use crate::Entry;

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

impl Colorizer {
    pub fn file(&self, entry: &Entry) -> Style {
        for m in self.group_styles.iter() {
            if m.matches(entry) {
                return m.style()
            }
        }
        Style::default()
    }

    pub fn file_size(&self) -> Style {
        Style::default().fg::<Gray>()
    }

    pub fn date_modified(&self) -> Style {
        Style::default().blue()
    }
}
