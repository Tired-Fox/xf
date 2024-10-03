use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use regex::Regex;

#[derive(Default, Debug, Clone)]
pub struct GitIgnore {
    include: Vec<PathBuf>,
    exclude: Vec<Regex>,
}

impl GitIgnore {
    pub fn include(&self, path: impl AsRef<Path>) -> bool {
        if self.include.contains(&path.as_ref().to_path_buf()) {
            return true;
        }

        let mut path = path.as_ref().display().to_string().replace("\\", "/");
        if path.starts_with("/") {
            path = path.strip_prefix('/').unwrap().to_string();
        }

        if path.ends_with("/") {
            path = path.strip_suffix('/').unwrap().to_string();
        }

        for exclude in self.exclude.iter() {
            if exclude.is_match(path.as_str()) {
                return false;
            }
        }

        true
    }
}

impl TryFrom<PathBuf> for GitIgnore {
    type Error = String;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let content = std::fs::read_to_string(value).map_err(|e| e.to_string())?;
        Self::from_str(content.as_str())
    }
}

impl FromStr for GitIgnore {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ignore = GitIgnore::default();

        for line in s.lines() {
            let mut line = line.trim().to_string();

            if line.is_empty() || line.starts_with("#") {
                continue;
            } else if line.starts_with("!") {
                ignore
                    .include
                    .push(PathBuf::from(line.strip_prefix('!').unwrap()));
            } else {
                line = line
                    .replace(".", "\\.")
                    .replace("**", ".*")
                    .replace("*", r"[^/\\]+");

                if line.starts_with("/") {
                    line = line.strip_prefix('/').unwrap().to_string();
                }

                if line.ends_with("/") {
                    line = line.strip_suffix('/').unwrap().to_string();
                }

                ignore.exclude.push(
                    Regex::new(format!("^{}$", line.as_str()).as_str())
                        .map_err(|e| e.to_string())?,
                )
            }
        }

        Ok(ignore)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_git_ignore() {
        let ignore = GitIgnore::from_str("**/test.txt");
        assert!(ignore.is_ok());
        assert_eq!(ignore.unwrap().exclude.len(), 1);

        let ignore = GitIgnore::from_str("target/*");
        assert!(ignore.is_ok());
        assert_eq!(ignore.unwrap().exclude.len(), 1);

        let ignore = GitIgnore::from_str("*.txt");
        assert!(ignore.is_ok());
        assert_eq!(ignore.unwrap().exclude.len(), 1);

        let ignore = GitIgnore::from_str("!test.txt");
        assert!(ignore.is_ok());
        let ignore = ignore.unwrap();
        assert_eq!(ignore.exclude.len(), 0);
        assert_eq!(ignore.include.len(), 1);

        let ignore = GitIgnore::from_str("# test.txt");
        assert!(ignore.is_ok());
        let ignore = ignore.unwrap();
        assert_eq!(ignore.exclude.len(), 0);
        assert_eq!(ignore.include.len(), 0);
    }

    #[test]
    fn should_include() {
        let ignore = GitIgnore::from_str(
            "
**/test.rs
*.zip
# comment
tests/**/*.log
!examples/test.rs
",
        )
        .unwrap();

        assert!(ignore.include("examples/test.rs"));
        assert!(!ignore.include("compressed.zip"));
        assert!(!ignore.include("tests/nested/output.log"));
        assert!(!ignore.include("tests/test.rs"));
    }
}
