use regex::Regex;
use std::path::Path;

pub enum IgnoreRule {
    Directory(String),
    Glob {
        regex: Regex,
        match_components: bool,
    },
    Literal {
        value: String,
        match_components: bool,
    },
}

impl IgnoreRule {
    pub fn from_pattern(pattern: &str) -> Option<Self> {
        let pattern = pattern.trim();
        if pattern.is_empty() || pattern.starts_with('#') {
            return None;
        }

        if pattern.ends_with('/') {
            let dir = pattern.trim_end_matches('/').trim();
            if dir.is_empty() {
                return None;
            }
            return Some(IgnoreRule::Directory(dir.to_string()));
        }

        if pattern.contains('*') || pattern.contains('?') {
            let regex = glob_to_regex(pattern).ok()?;
            return Some(IgnoreRule::Glob {
                regex,
                match_components: !pattern.contains('/'),
            });
        }

        Some(IgnoreRule::Literal {
            value: pattern.to_string(),
            match_components: !pattern.contains('/'),
        })
    }

    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        match self {
            IgnoreRule::Directory(dir) => path
                .components()
                .any(|component| component.as_os_str().to_string_lossy().as_ref() == dir),
            IgnoreRule::Glob {
                regex,
                match_components,
            } => {
                if *match_components {
                    path.components().any(|component| {
                        regex.is_match(component.as_os_str().to_string_lossy().as_ref())
                    })
                } else {
                    let normalized = path_str.replace('\\', "/");
                    if regex.is_match(&normalized) {
                        return true;
                    }

                    let mut suffix = normalized.trim_start_matches('/');
                    loop {
                        if regex.is_match(suffix) {
                            return true;
                        }

                        if let Some(pos) = suffix.find('/') {
                            suffix = &suffix[pos + 1..];
                        } else {
                            break;
                        }
                    }

                    false
                }
            }
            IgnoreRule::Literal {
                value,
                match_components,
            } => {
                if *match_components {
                    path.components()
                        .any(|component| component.as_os_str().to_string_lossy().as_ref() == value)
                } else {
                    path_str.contains(value)
                }
            }
        }
    }
}

fn glob_to_regex(glob: &str) -> Result<Regex, regex::Error> {
    let mut regex = String::from("^");
    let mut chars = glob.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    regex.push_str(".*");
                } else {
                    regex.push_str("[^/]*");
                }
            }
            '?' => regex.push_str("[^/]"),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }

    regex.push('$');
    Regex::new(&regex)
}

pub fn load_ignore_patterns() -> Vec<IgnoreRule> {
    let paths = [
        std::path::PathBuf::from(".scpfignore"),
        dirs::home_dir()
            .map(|h| h.join(".scpf/ignore"))
            .unwrap_or_default(),
    ];

    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            return content
                .lines()
                .filter_map(IgnoreRule::from_pattern)
                .collect();
        }
    }

    // Default ignore patterns
    vec![
        IgnoreRule::from_pattern("node_modules/"),
        IgnoreRule::from_pattern("target/"),
        IgnoreRule::from_pattern(".git/"),
        IgnoreRule::from_pattern("*.test.sol"),
        IgnoreRule::from_pattern("test/"),
        IgnoreRule::from_pattern("tests/"),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_extension_patterns_do_not_overmatch_directory_names() {
        let rule = IgnoreRule::from_pattern("*.test.sol").unwrap();
        assert!(!rule.matches(Path::new("/tmp/test")));
        assert!(rule.matches(Path::new("/tmp/foo.test.sol")));
    }

    #[test]
    fn doublestar_glob_matches_nested_paths() {
        let rule = IgnoreRule::from_pattern("tests/**/*.sol").unwrap();
        assert!(rule.matches(Path::new("/tmp/tests/a/b.sol")));
        assert!(!rule.matches(Path::new("/tmp/tests/a/b.txt")));
    }

    #[test]
    fn directory_patterns_match_any_component() {
        let rule = IgnoreRule::from_pattern("node_modules/").unwrap();
        assert!(rule.matches(Path::new("/tmp/node_modules/foo")));
        assert!(rule.matches(Path::new("/tmp/a/node_modules/foo")));
    }
}
