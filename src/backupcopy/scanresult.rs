use std::{collections::HashMap, path::PathBuf};

use super::fsitem::FsItem;

#[derive(Debug)]
pub struct ScanResult {
    pub base: PathBuf,
    pub data: HashMap<PathBuf, FsItem>,
}

impl ScanResult {
    pub fn new(base: PathBuf) -> ScanResult {
        ScanResult {
            base,
            data: HashMap::new(),
        }
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
        self.data.insert(relative, file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_are_keyed_by_relative_path() {
        let mut r = ScanResult::new(PathBuf::from(r"C:\src"));
        r.add_dir(PathBuf::from(r"C:\src\sub"));
        r.add_file(PathBuf::from(r"C:\src\sub\a.txt"), 3, 7);

        let dir = &r.data[&PathBuf::from("sub")];
        assert!(dir.is_directory());
        assert_eq!(dir.get_full_path(), &PathBuf::from(r"C:\src\sub"));

        let file = &r.data[&PathBuf::from(r"sub\a.txt")];
        assert!(file.is_file());
        assert_eq!(file.get_size(), 3);
        assert_eq!(file.get_date(), 7);
        assert_eq!(file.get_full_path(), &PathBuf::from(r"C:\src\sub\a.txt"));
    }

    #[test]
    fn path_outside_base_is_kept_as_is() {
        let mut r = ScanResult::new(PathBuf::from(r"C:\src"));
        r.add_file(PathBuf::from(r"D:\other\a.txt"), 1, 1);
        assert!(r.data.contains_key(&PathBuf::from(r"D:\other\a.txt")));
    }

    #[test]
    fn base_prefix_is_matched_by_component() {
        let mut r = ScanResult::new(PathBuf::from(r"C:\src"));
        r.add_file(PathBuf::from(r"C:\srcx\a.txt"), 1, 1);
        assert!(r.data.contains_key(&PathBuf::from(r"C:\srcx\a.txt")));
    }
}
