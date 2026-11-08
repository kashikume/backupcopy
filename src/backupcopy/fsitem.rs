use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlannedAction {
    Unknown,
    Ignore,
    Copy,
    Delete,
}

#[derive(Debug)]
pub struct FsDirectory {
    pub full_path: PathBuf,
    pub action: PlannedAction,
}

#[derive(Debug)]
pub struct FsFile {
    pub full_path: PathBuf,
    pub size: u64,
    pub date: u64,
    pub action: PlannedAction,
}

#[derive(Debug)]
pub enum FsItem {
    Directory(FsDirectory),
    File(FsFile),
}

impl FsItem {
    pub fn new_directory(full_path: PathBuf) -> Self {
        FsItem::Directory(FsDirectory {
            full_path,
            action: PlannedAction::Unknown,
        })
    }

    pub fn new_file(full_path: PathBuf, size: u64, date: u64) -> Self {
        FsItem::File(FsFile {
            full_path,
            size,
            date,
            action: PlannedAction::Unknown,
        })
    }

    pub fn set_action(&mut self, action: PlannedAction) {
        match self {
            Self::Directory(dir) => dir.action = action,
            Self::File(file) => file.action = action,
        }
    }

    pub fn get_action(&self) -> PlannedAction {
        match self {
            Self::Directory(dir) => dir.action,
            Self::File(file) => file.action,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Directory(_) => false,
            Self::File(_) => true,
        }
    }

    pub fn is_directory(&self) -> bool {
        match self {
            Self::Directory(_) => true,
            Self::File(_) => false,
        }
    }

    pub fn get_full_path(&self) -> &PathBuf {
        match self {
            Self::Directory(dir) => &dir.full_path,
            Self::File(file) => &file.full_path,
        }
    }

    pub fn get_size(&self) -> u64 {
        match self {
            Self::Directory(_) => panic!("Directory do not have size."),
            Self::File(file) => file.size,
        }
    }

    pub fn get_date(&self) -> u64 {
        match self {
            Self::Directory(_) => panic!("Directory do not have date."),
            Self::File(file) => file.date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_items_have_unknown_action() {
        assert_eq!(
            FsItem::new_directory(PathBuf::from("d")).get_action(),
            PlannedAction::Unknown
        );
        assert_eq!(
            FsItem::new_file(PathBuf::from("f"), 1, 2).get_action(),
            PlannedAction::Unknown
        );
    }

    #[test]
    fn file_accessors() {
        let mut f = FsItem::new_file(PathBuf::from(r"C:\a.txt"), 10, 20);
        assert!(f.is_file());
        assert!(!f.is_directory());
        assert_eq!(f.get_size(), 10);
        assert_eq!(f.get_date(), 20);
        assert_eq!(f.get_full_path(), &PathBuf::from(r"C:\a.txt"));
        f.set_action(PlannedAction::Copy);
        assert_eq!(f.get_action(), PlannedAction::Copy);
    }

    #[test]
    fn directory_accessors() {
        let mut d = FsItem::new_directory(PathBuf::from(r"C:\dir"));
        assert!(d.is_directory());
        assert!(!d.is_file());
        assert_eq!(d.get_full_path(), &PathBuf::from(r"C:\dir"));
        d.set_action(PlannedAction::Delete);
        assert_eq!(d.get_action(), PlannedAction::Delete);
    }

    #[test]
    #[should_panic]
    fn directory_has_no_size() {
        FsItem::new_directory(PathBuf::from("d")).get_size();
    }

    #[test]
    #[should_panic]
    fn directory_has_no_date() {
        FsItem::new_directory(PathBuf::from("d")).get_date();
    }
}
