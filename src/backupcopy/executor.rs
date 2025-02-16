use std::fs;
use std::{iter::Scan, path::PathBuf};

use super::{fsitem::PlannedAction, scanresult::ScanResult};
use anyhow::Result;

pub struct Executor {}

impl Executor {
    fn remove_files(destination: &ScanResult) -> Result<()> {
        let files_to_remove = destination
            .data
            .iter()
            .filter(|(_k, d)| d.is_file() && d.get_action() == PlannedAction::Delete)
            .map(|(_k, d)| d.get_full_path())
            .collect::<Vec<_>>();
        println!("Files to remove: {:?}", files_to_remove);
        Ok(())
    }

    fn remove_directories(destination: &ScanResult) -> Result<()> {
        let dirs_to_remove = destination
            .data
            .iter()
            .filter(|(k, d)| d.is_directory() && d.get_action() == PlannedAction::Delete)
            .map(|(k, d)| d.get_full_path())
            .collect::<Vec<_>>();
        println!("Directories to remove: {:?}", dirs_to_remove);
        Ok(())
    }

    fn create_directories(source: &ScanResult, base: &PathBuf) -> Result<()> {
        let mut dirs_to_create = source
            .data
            .iter()
            .filter(|(_k, d)| d.is_directory() && d.get_action() == PlannedAction::Copy)
            .map(|(k, _d)| {
                let mut o = base.clone();
                o.push(k);
                o
            })
            .collect::<Vec<_>>();

        dirs_to_create.sort();

        println!("Directories to create: {:?}", dirs_to_create);
        for d in dirs_to_create.iter() {
            println!("Create directory: {:?}", d);
            fs::create_dir(d)?;
        }

        Ok(())
    }

    fn copy_files(source: &ScanResult, base: &PathBuf) -> Result<()> {
        let files_to_copy = source
            .data
            .iter()
            .filter(|(k, f)| f.is_file() && f.get_action() == PlannedAction::Copy)
            .collect::<Vec<_>>();

        for (k, f) in files_to_copy {
            let copy_from = f.get_full_path();
            let mut copy_to = base.clone();
            copy_to.push(k);
            println!("Copy {:?} to {:?}", copy_from, copy_to);
            fs::copy(copy_from, copy_to)?;
        }

        Ok(())
    }

    pub fn execute(source: &ScanResult, destination: &ScanResult) -> Result<()> {
        Self::remove_files(destination)?;
        Self::remove_directories(destination)?;
        Self::create_directories(source, &destination.base)?;
        Self::copy_files(source, &destination.base)?;
        Ok(())
    }
}
