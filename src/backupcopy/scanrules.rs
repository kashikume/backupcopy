use std::path::PathBuf;

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

    pub fn add_rules(&mut self, file: &PathBuf) {}
}
