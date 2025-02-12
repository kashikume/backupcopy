use std::path::PathBuf;
use std::fs;
use anyhow::Result;

#[derive(Debug)]
pub struct ScanRules {
    data: Vec<Rule>,
}

#[derive(Debug)]
enum RuleAction {
    Keep,
    Remove,
}

#[derive(Debug)]
struct Rule {
    dir: PathBuf,
    regex: String,
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
            }
            else if line.starts_with("skip:") || line.starts_with("remove:") || line.starts_with("-:") {
                let start = line.find(':').unwrap();
                let regex = line[start+1..].trim();
                let rule = Rule{
                    dir: file.parent().unwrap().to_path_buf(),
                    regex: String::from(regex),
                    action: RuleAction::Remove,
                };
                self.data.push(rule);
            }
            else if line.starts_with("keep:") || line.starts_with("add:") || line.starts_with("+:") {
                let start = line.find(':').unwrap();
                let regex = line[start+1..].trim();
                let rule = Rule{
                    dir: file.parent().unwrap().to_path_buf(),
                    regex: String::from(regex),
                    action: RuleAction::Keep,
                };
                self.data.push(rule);
            }
        }

        Ok(())
    }
}
