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
}
