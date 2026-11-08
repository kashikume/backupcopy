use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ScanRules {
    data: Vec<Rule>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RuleAction {
    Keep,
    Remove,
}

#[derive(Debug)]
struct Rule {
    dir: PathBuf,
    regex: Regex,
    action: RuleAction,
}

impl ScanRules {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn add_rules(&mut self, file: &PathBuf) -> Result<()> {
        let content = fs::read_to_string(file)
            .with_context(|| format!("Cannot read rules file {:?}", file))?;
        self.add_rules_from_str(file, &content)
    }

    /// Parses rules from `content` as if they were read from `file`.
    pub fn add_rules_from_str(&mut self, file: &PathBuf, content: &str) -> Result<()> {
        for (index, line) in content.lines().enumerate() {
            let line = line.trim();
            let action = if line.is_empty() || line.starts_with('#') {
                continue;
            } else if line.starts_with("skip:")
                || line.starts_with("remove:")
                || line.starts_with("-:")
            {
                RuleAction::Remove
            } else if line.starts_with("keep:")
                || line.starts_with("add:")
                || line.starts_with("+:")
            {
                RuleAction::Keep
            } else {
                Self::print_file_with_error(file, &content, index);
                return Err(anyhow!(
                    "Invalid rule in rules file {:?}, line {}: {}\n\
                     A rule must start with one of: skip:, remove:, -:, keep:, add:, +:",
                    file,
                    index + 1,
                    line
                ));
            };

            let start = line.find(':').unwrap();
            let regex = line[start + 1..].trim();
            let regex = match Regex::new(regex) {
                Ok(regex) => regex,
                Err(e) => {
                    Self::print_file_with_error(file, &content, index);
                    return Err(anyhow!(
                        "Invalid regular expression in rules file {:?}, line {}: {}\n{}",
                        file,
                        index + 1,
                        line,
                        e
                    ));
                }
            };
            self.data.push(Rule {
                dir: file.parent().unwrap().to_path_buf(),
                regex,
                action,
            });
        }

        self.data.sort_by(|a, b| {
            if a.action == b.action {
                return a.dir.cmp(&b.dir);
            }
            if a.action == RuleAction::Keep {
                return std::cmp::Ordering::Less;
            }
            std::cmp::Ordering::Greater
        });

        Ok(())
    }

    fn print_file_with_error(file: &PathBuf, content: &str, error_index: usize) {
        println!("Error in rules file {:?}:", file);
        for (index, line) in content.lines().enumerate() {
            let marker = if index == error_index { ">>" } else { "  " };
            println!("{} {:4} | {}", marker, index + 1, line);
        }
    }

    pub fn evaluate(&self, path: &PathBuf) -> RuleAction {
        let mut action = None;
        for r in self.data.iter() {
            if path.starts_with(&r.dir) {
                let p1: PathBuf = path.iter().skip(r.dir.iter().count()).collect();
                let s = p1.to_str().unwrap();
                if r.regex.is_match(&s) {
                    action = Some(r.action);
                    if r.action == RuleAction::Keep {
                        break;
                    }
                }
            }
        }

        if action.is_none() {
            action = Some(RuleAction::Keep);
        }

        action.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(file: &str, content: &str) -> ScanRules {
        let mut r = ScanRules::new();
        r.add_rules_from_str(&PathBuf::from(file), content).unwrap();
        r
    }

    fn eval(r: &ScanRules, path: &str) -> RuleAction {
        r.evaluate(&PathBuf::from(path))
    }

    #[test]
    fn no_rules_keeps_everything() {
        let r = ScanRules::new();
        assert_eq!(eval(&r, r"C:\src\a.txt"), RuleAction::Keep);
    }

    #[test]
    fn all_remove_prefixes() {
        for prefix in ["skip:", "remove:", "-:"] {
            let r = rules(r"C:\src\.backup_rules", &format!(r"{} \.tmp$", prefix));
            assert_eq!(eval(&r, r"C:\src\a.tmp"), RuleAction::Remove, "{}", prefix);
            assert_eq!(eval(&r, r"C:\src\a.txt"), RuleAction::Keep, "{}", prefix);
        }
    }

    #[test]
    fn all_keep_prefixes_override_remove() {
        for prefix in ["keep:", "add:", "+:"] {
            let r = rules(
                r"C:\src\.backup_rules",
                &format!("-: \\.log$\n{} ^important\\.log$", prefix),
            );
            assert_eq!(eval(&r, r"C:\src\important.log"), RuleAction::Keep, "{}", prefix);
            assert_eq!(eval(&r, r"C:\src\other.log"), RuleAction::Remove, "{}", prefix);
        }
    }

    #[test]
    fn keep_wins_regardless_of_order_and_file() {
        let mut r = ScanRules::new();
        r.add_rules_from_str(&PathBuf::from(r"C:\src\sub\.backup_rules"), "-: .*")
            .unwrap();
        r.add_rules_from_str(&PathBuf::from(r"C:\src\.backup_rules"), "+: keep")
            .unwrap();
        assert_eq!(eval(&r, r"C:\src\sub\keep.txt"), RuleAction::Keep);
        assert_eq!(eval(&r, r"C:\src\sub\other.txt"), RuleAction::Remove);
    }

    #[test]
    fn comments_blank_lines_and_whitespace() {
        let r = rules(
            r"C:\src\.backup_rules",
            "# comment\n\n   \n   -:    \\.tmp$   \r\n",
        );
        assert_eq!(eval(&r, r"C:\src\a.tmp"), RuleAction::Remove);
        assert_eq!(eval(&r, r"C:\src\a.txt"), RuleAction::Keep);
    }

    #[test]
    fn regex_is_matched_against_path_relative_to_rules_dir() {
        let r = rules(r"C:\src\.backup_rules", r"-: ^build(\\|$)");
        assert_eq!(eval(&r, r"C:\src\build"), RuleAction::Remove);
        assert_eq!(eval(&r, r"C:\src\build\app.exe"), RuleAction::Remove);
        assert_eq!(eval(&r, r"C:\src\sub\build"), RuleAction::Keep);
        assert_eq!(eval(&r, r"C:\src\buildscript.rs"), RuleAction::Keep);
    }

    #[test]
    fn unanchored_regex_matches_anywhere() {
        let r = rules(r"C:\src\.backup_rules", "-: tmp");
        assert_eq!(eval(&r, r"C:\src\dir\mytmpfile.rs"), RuleAction::Remove);
    }

    #[test]
    fn rules_apply_only_below_their_directory() {
        let r = rules(r"C:\src\sub\.backup_rules", "-: .*");
        assert_eq!(eval(&r, r"C:\src\sub\a.txt"), RuleAction::Remove);
        assert_eq!(eval(&r, r"C:\src\other\a.txt"), RuleAction::Keep);
        assert_eq!(eval(&r, r"C:\src\a.txt"), RuleAction::Keep);
    }

    #[test]
    fn rules_dir_prefix_is_matched_by_component() {
        let r = rules(r"C:\src\sub\.backup_rules", "-: .*");
        assert_eq!(eval(&r, r"C:\src\subway\a.txt"), RuleAction::Keep);
    }

    #[test]
    fn invalid_regex_is_an_error() {
        let mut r = ScanRules::new();
        let e = r
            .add_rules_from_str(&PathBuf::from(r"C:\src\.backup_rules"), "# c\n-: (unclosed")
            .unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("Invalid regular expression"), "{}", msg);
        assert!(msg.contains("line 2"), "{}", msg);
    }

    #[test]
    fn unknown_prefix_is_an_error() {
        let mut r = ScanRules::new();
        let e = r
            .add_rules_from_str(&PathBuf::from(r"C:\src\.backup_rules"), "-: ok\nskp: x")
            .unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("Invalid rule"), "{}", msg);
        assert!(msg.contains("line 2"), "{}", msg);
        assert!(msg.contains("skp: x"), "{}", msg);
    }

    #[test]
    fn line_without_colon_is_an_error() {
        let mut r = ScanRules::new();
        assert!(r
            .add_rules_from_str(&PathBuf::from(r"C:\src\.backup_rules"), "skip")
            .is_err());
    }

    #[test]
    fn missing_rules_file_is_an_error() {
        let mut r = ScanRules::new();
        let e = r
            .add_rules(&PathBuf::from(r"C:\does\not\exist\.backup_rules"))
            .unwrap_err();
        assert!(e.to_string().contains("Cannot read rules file"));
    }
}
