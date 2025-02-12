use std::{collections::HashMap, path::PathBuf};

use super::fsitem::FsItem;

#[derive(Debug)]
pub struct ScanResult {
    base: PathBuf,
    data: HashMap<PathBuf,FsItem>,
}

impl ScanResult {
    pub fn new(base: PathBuf) -> ScanResult {
        ScanResult { base, data: HashMap::new() }
    }

    fn get_relative(&self, path: &PathBuf) -> PathBuf {
        if path.starts_with(&self.base) {
            let len = self.base.iter().count();
            path.iter().skip(len).collect()
        } else {
            path.clone()
        }
    }

    pub fn add_dir(&mut self, path: PathBuf) {
        let relative = self.get_relative(&path);
        let dir = FsItem::new_directory(path);
        self.data.insert(relative, dir);
    }

    pub fn add_file(&mut self, path: PathBuf, size: u64, date: u64) {
        let relative = self.get_relative(&path);
        let file = FsItem::new_file(path, size, date);
        self.data.insert(relative,file);
    }
}
