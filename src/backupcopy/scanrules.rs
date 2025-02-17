use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use regex::Regex;

#[derive(Debug)]
pub struct ScanRules {
    data: Vec<Rule>,
}

#[derive(Copy, Clone, Debug)]
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
        let content = fs::read_to_string(file)?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            } else if line.starts_with("skip:")
                || line.starts_with("remove:")
                || line.starts_with("-:")
            {
                let start = line.find(':').unwrap();
                let regex = line[start + 1..].trim();
                let rule = Rule {
                    dir: file.parent().unwrap().to_path_buf(),
                    regex: Regex::new(regex)?,
                    action: RuleAction::Remove,
                };
                self.data.push(rule);
            } else if line.starts_with("keep:")
                || line.starts_with("add:")
                || line.starts_with("+:")
            {
                let start = line.find(':').unwrap();
                let regex = line[start + 1..].trim();
                let rule = Rule {
                    dir: file.parent().unwrap().to_path_buf(),
                    regex: Regex::new(regex)?,
                    action: RuleAction::Keep,
                };
                self.data.push(rule);
            }
        }

        Ok(())
    }

    pub fn evaluate(&self, path: &PathBuf) -> RuleAction {

        for r in  self.data.iter() {
            if path.starts_with(&r.dir) {
                let p1: PathBuf = path.iter().skip(r.dir.iter().count()).collect();
                let s = p1.to_str().unwrap();
                if r.regex.is_match(&s) {
                    return r.action;
                }
            }
        }

        RuleAction::Keep
    }
}
