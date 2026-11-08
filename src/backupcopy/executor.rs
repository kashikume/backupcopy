use std::fs;
use std::path::PathBuf;

use super::{arguments::Arguments, fsitem::PlannedAction, scanresult::ScanResult};
use anyhow::Result;

pub struct Executor {}

impl Executor {
    fn remove_files(destination: &ScanResult, arguments: &Arguments) -> Result<()> {
        let files_to_remove = destination
            .data
            .iter()
            .filter(|(_k, d)| d.is_file() && d.get_action() == PlannedAction::Delete)
            .map(|(_k, d)| d.get_full_path())
            .collect::<Vec<_>>();
        // println!("Files to remove: {:?}", files_to_remove);
        for f in files_to_remove {
            print!("Remove: {:?} ... ", f);
            if arguments.remove_ro {
                let mut perms = fs::metadata(&f)?.permissions();
                if perms.readonly() {
                    print!("remove readonly flag ... ");
                    perms.set_readonly(false);
                    match fs::set_permissions(&f, perms) {
                        Ok(_) => (),
                        Err(e) => println!("Error: {:?}", e),
                    }
                }
            }
            match fs::remove_file(f) {
                Ok(_) => println!("OK"),
                Err(e) => println!("Error: {:?}", e),
            }
        }
        Ok(())
    }

    fn remove_directories(destination: &ScanResult) -> Result<()> {
        let mut dirs_to_remove = destination
            .data
            .iter()
            .filter(|(_k, d)| d.is_directory() && d.get_action() == PlannedAction::Delete)
            .map(|(_k, d)| d.get_full_path())
            .collect::<Vec<_>>();
        // println!("Directories to remove: {:?}", dirs_to_remove);

        dirs_to_remove.sort_by(|a, b| b.as_os_str().len().cmp(&a.as_os_str().len()));

        for d in dirs_to_remove {
            print!("Remove: {:?} ... ", d);
            match fs::remove_dir(d) {
                Ok(_) => println!("OK"),
                Err(e) => println!("Error: {:?}", e),
            }
        }

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

        // println!("Directories to create: {:?}", dirs_to_create);
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
            .filter(|(_k, f)| f.is_file() && f.get_action() == PlannedAction::Copy)
            .collect::<Vec<_>>();

        let mut counter = 1usize;
        let total = files_to_copy.len();
        for (k, f) in files_to_copy {
            let copy_from = f.get_full_path();
            let mut copy_to = base.clone();
            copy_to.push(k);
            print!("{}/{} Copy {:?} to {:?}", counter, total, copy_from, copy_to);
            match fs::copy(copy_from, copy_to) {
                Ok(_) => println!(" ... OK"),
                Err(e) => println!(" ... Error: {:?}", e),
            }
            counter += 1;
        }

        Ok(())
    }

    pub fn execute(
        source: &ScanResult,
        destination: &ScanResult,
        arguments: &Arguments,
    ) -> Result<()> {
        Self::remove_files(destination, arguments)?;
        Self::remove_directories(destination)?;
        Self::create_directories(source, &destination.base)?;
        Self::copy_files(source, &destination.base)?;
        Ok(())
    }
}

/// End-to-end tests: scan, plan and execute on real directories.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backupcopy::analyzer::Analyzer;
    use crate::backupcopy::fsscanner::FsScanner;
    use crate::backupcopy::testutil::{set_readonly, TestDir};
    use std::fs::File;

    fn sync(src: &str, dst: &str, remove_ro: bool) -> Result<()> {
        let args = Arguments {
            source: src.to_string(),
            destination: dst.to_string(),
            verbose: false,
            debug: false,
            remove_ro,
        };
        let (mut s, rules) = FsScanner::scan(&args.source)?;
        let (mut d, _) = FsScanner::scan(&args.destination)?;
        Analyzer::plan_actions(&mut s, &mut d, &rules);
        Executor::execute(&s, &d, &args)
    }

    /// Lists every file and directory under `root` as sorted relative paths;
    /// directories end with a backslash, files are followed by `=content`.
    fn tree(root: &PathBuf) -> Vec<String> {
        fn walk(root: &PathBuf, dir: &PathBuf, out: &mut Vec<String>) {
            for entry in fs::read_dir(dir).unwrap() {
                let p = entry.unwrap().path();
                let rel = p.strip_prefix(root).unwrap().to_str().unwrap().to_string();
                if p.is_dir() {
                    out.push(format!("{}\\", rel));
                    walk(root, &p, out);
                } else {
                    out.push(format!("{}={}", rel, fs::read_to_string(&p).unwrap()));
                }
            }
        }
        let mut out = vec![];
        walk(root, root, &mut out);
        out.sort();
        out
    }

    struct Fixture {
        t: TestDir,
        src: PathBuf,
        dst: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let t = TestDir::new(name);
            let src = t.dir("src");
            let dst = t.dir("dst");
            Fixture { t, src, dst }
        }

        fn sync(&self, remove_ro: bool) -> Result<()> {
            sync(&self.t.path_string("src"), &self.t.path_string("dst"), remove_ro)
        }
    }

    #[test]
    fn copies_full_tree_into_empty_destination() {
        let f = Fixture::new("full");
        f.t.file(r"src\a.txt", "a");
        f.t.file(r"src\sub\b.txt", "b");
        f.t.file(r"src\sub\deep\c.txt", "c");
        f.t.dir(r"src\empty");

        f.sync(false).unwrap();

        assert_eq!(tree(&f.dst), tree(&f.src));
    }

    #[test]
    fn removes_items_deleted_from_source() {
        let f = Fixture::new("removed");
        f.t.file(r"src\keep.txt", "k");
        f.t.file(r"src\gone.txt", "g");
        f.t.file(r"src\olddir\deep\x.txt", "x");
        f.sync(false).unwrap();

        fs::remove_file(f.src.join("gone.txt")).unwrap();
        fs::remove_dir_all(f.src.join("olddir")).unwrap();
        f.sync(false).unwrap();

        assert_eq!(tree(&f.dst), vec!["keep.txt=k"]);
    }

    #[test]
    fn replaces_changed_files() {
        let f = Fixture::new("changed");
        f.t.file(r"src\a.txt", "old");
        f.sync(false).unwrap();

        f.t.file(r"src\a.txt", "new content");
        f.sync(false).unwrap();

        assert_eq!(tree(&f.dst), vec!["a.txt=new content"]);
    }

    #[test]
    fn copy_keeps_modification_time_so_unchanged_files_are_not_copied_again() {
        let f = Fixture::new("unchanged");
        f.t.file(r"src\a.txt", "aaa");
        f.sync(false).unwrap();

        // Same size and same time: a second run must leave the destination file alone.
        let modified = fs::metadata(f.src.join("a.txt")).unwrap().modified().unwrap();
        fs::write(f.dst.join("a.txt"), "zzz").unwrap();
        File::options()
            .write(true)
            .open(f.dst.join("a.txt"))
            .unwrap()
            .set_modified(modified)
            .unwrap();
        f.sync(false).unwrap();

        assert_eq!(tree(&f.dst), vec!["a.txt=zzz"]);
    }

    #[test]
    fn replaces_file_with_directory_and_back() {
        let f = Fixture::new("typechange");
        f.t.file(r"src\x", "file");
        f.t.dir(r"src\y");
        f.t.file(r"src\y\inner.txt", "i");
        f.sync(false).unwrap();

        fs::remove_file(f.src.join("x")).unwrap();
        f.t.file(r"src\x\inner.txt", "now a dir");
        fs::remove_dir_all(f.src.join("y")).unwrap();
        f.t.file(r"src\y", "now a file");
        f.sync(false).unwrap();

        assert_eq!(tree(&f.dst), tree(&f.src));
    }

    #[test]
    fn applies_rules_files() {
        let f = Fixture::new("rules");
        f.t.file(r"src\.backup_rules", "-: (^|\\\\)build(\\\\|$)\n-: \\.tmp$\n+: ^keep\\.tmp$");
        f.t.file(r"src\a.txt", "a");
        f.t.file(r"src\a.tmp", "t");
        f.t.file(r"src\keep.tmp", "k");
        f.t.file(r"src\build\out.bin", "o");
        f.t.file(r"src\proj\build\out.bin", "o");
        f.t.file(r"src\proj\main.rs", "m");
        f.t.file(r"src\proj\.backup_rules", "-: ^secret");
        f.t.file(r"src\proj\secret.txt", "s");
        f.t.file(r"src\secret.txt", "not excluded here");

        f.sync(false).unwrap();

        assert_eq!(
            tree(&f.dst),
            vec![
                r".backup_rules=-: (^|\\)build(\\|$)
-: \.tmp$
+: ^keep\.tmp$",
                "a.txt=a",
                "keep.tmp=k",
                r"proj\",
                r"proj\.backup_rules=-: ^secret",
                r"proj\main.rs=m",
                "secret.txt=not excluded here",
            ]
        );
    }

    #[test]
    fn removes_items_that_a_new_rule_excludes() {
        let f = Fixture::new("newrule");
        f.t.file(r"src\a.txt", "a");
        f.t.file(r"src\cache\c.bin", "c");
        f.sync(false).unwrap();

        f.t.file(r"src\.backup_rules", r"-: ^cache(\\|$)");
        f.sync(false).unwrap();

        assert_eq!(
            tree(&f.dst),
            vec![r".backup_rules=-: ^cache(\\|$)", "a.txt=a"]
        );
    }

    #[test]
    fn read_only_files_are_removed_with_remove_ro() {
        // Whether a read-only file can be deleted without --remove-ro depends on the
        // Rust standard library version, so only the --remove-ro case is checked.
        let f = Fixture::new("readonly");
        f.t.file(r"src\a.txt", "a");
        f.t.file(r"src\changed.txt", "old");
        let extra = f.t.file(r"dst\extra.txt", "e");
        set_readonly(&extra, true);
        f.sync(true).unwrap();

        f.t.file(r"src\changed.txt", "new content");
        set_readonly(&f.dst.join("changed.txt"), true);
        f.sync(true).unwrap();

        assert_eq!(tree(&f.dst), vec!["a.txt=a", "changed.txt=new content"]);
    }

    #[test]
    fn invalid_rules_file_stops_before_any_change() {
        let f = Fixture::new("badrules");
        f.t.file(r"src\a.txt", "a");
        f.t.file(r"src\.backup_rules", "skp: x");
        f.t.file(r"dst\old.txt", "o");

        assert!(f.sync(false).is_err());
        assert_eq!(tree(&f.dst), vec!["old.txt=o"]);
    }

    #[test]
    fn missing_source_or_destination_is_an_error() {
        let f = Fixture::new("missing");
        assert!(sync(&f.t.path_string("nope"), &f.t.path_string("dst"), false).is_err());
        assert!(sync(&f.t.path_string("src"), &f.t.path_string("nope"), false).is_err());
    }
}
