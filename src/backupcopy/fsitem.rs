use std::path::PathBuf;

#[derive(Debug)]
pub enum PlannedAction {
    Unknown,
    Ignore,
    Copy,
    Delete,
}

#[derive(Debug)]
pub struct FsDirectory {
    pub path: PathBuf,
    pub action: PlannedAction,
}

#[derive(Debug)]
pub struct FsFile {
    pub path: PathBuf,
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
    pub fn new_directory(path: PathBuf) -> Self {
        FsItem::Directory(FsDirectory {
            path,
            action: PlannedAction::Unknown,
        })
    }

    pub fn new_file(path: PathBuf, size: u64, date: u64) -> Self {
        FsItem::File(FsFile {
            path,
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

    pub fn get_patch(&self) -> &PathBuf {
        match self {
            Self::Directory(dir) => &dir.path,
            Self::File(file) => &file.path,
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
