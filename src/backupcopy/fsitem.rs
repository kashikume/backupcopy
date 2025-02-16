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
