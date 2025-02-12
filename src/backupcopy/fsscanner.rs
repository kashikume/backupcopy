use super::scanresult::ScanResult;
use super::scanrules::ScanRules;
use anyhow::Result;
use std::{fs, os::windows::fs::MetadataExt, path::PathBuf};

pub struct FsScanner {}

impl FsScanner {
    pub fn scan(dir: &String) -> Result<(ScanResult, ScanRules)> {
        let mut result = ScanResult::new(PathBuf::from(dir));
        let mut rules = ScanRules::new();

        let mut dirs = vec![PathBuf::from(dir)];

        while !dirs.is_empty() {
            let curr_dir = dirs.pop().unwrap();
            for entry in fs::read_dir(curr_dir)? {
                let entry = entry?;
                let path = entry.path();
                let metadata = entry.metadata()?;
                if metadata.is_dir() {
                    result.add_dir(path.clone());
                    dirs.push(path);
                } else if metadata.is_file() {
                    if path.file_name().unwrap() == ".backum_rules" {
                        rules.add_rules(&path);
                    }
                    result.add_file(path, metadata.len(), metadata.last_write_time());
                }
            }
        }

        Ok((result, rules))
    }
}
