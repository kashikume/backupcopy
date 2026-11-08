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
