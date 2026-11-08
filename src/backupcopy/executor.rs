use std::fs;
use std::path::PathBuf;

use super::{arguments::Arguments, fsitem::PlannedAction, scanresult::ScanResult};
use anyhow::{Context, Result};

/// What a run did, and how many single-item operations failed.
#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    pub files_deleted: usize,
    pub files_delete_failed: usize,
    pub dirs_deleted: usize,
    pub dirs_delete_failed: usize,
    pub dirs_created: usize,
    pub files_copied: usize,
    pub files_copy_failed: usize,
}

fn count(n: usize, singular: &str, plural: &str) -> String {
    format!("{} {}", n, if n == 1 { singular } else { plural })
}

impl Stats {
    pub fn errors(&self) -> usize {
        self.files_delete_failed + self.dirs_delete_failed + self.files_copy_failed
    }

    /// One-line summary, e.g. `copied 3 files, ...; 2 errors: 2 files failed to copy`.
    pub fn summary(&self) -> String {
        let done = format!(
            "copied {}, created {}, deleted {} and {}",
            count(self.files_copied, "file", "files"),
            count(self.dirs_created, "directory", "directories"),
            count(self.files_deleted, "file", "files"),
            count(self.dirs_deleted, "directory", "directories"),
        );

        let mut failures = vec![];
        if self.files_delete_failed > 0 {
            failures.push(format!(
                "{} failed to delete in destination",
                count(self.files_delete_failed, "file", "files")
            ));
        }
        if self.dirs_delete_failed > 0 {
            failures.push(format!(
                "{} failed to delete in destination",
                count(self.dirs_delete_failed, "directory", "directories")
            ));
        }
        if self.files_copy_failed > 0 {
            failures.push(format!(
                "{} failed to copy",
                count(self.files_copy_failed, "file", "files")
            ));
        }

        if failures.is_empty() {
            format!("{}; no errors", done)
        } else {
            format!(
                "{}; {}: {}",
                done,
                count(self.errors(), "error", "errors"),
                failures.join(", ")
            )
        }
    }
}

pub struct Executor {}

impl Executor {
    fn remove_files(destination: &ScanResult, arguments: &Arguments, stats: &mut Stats) {
        let files_to_remove = destination
            .data
            .iter()
            .filter(|(_k, d)| d.is_file() && d.get_action() == PlannedAction::Delete)
            .map(|(_k, d)| d.get_full_path())
            .collect::<Vec<_>>();
        for f in files_to_remove {
            print!("Remove: {:?} ... ", f);
            if arguments.remove_ro {
                if let Ok(metadata) = fs::metadata(f) {
                    let mut perms = metadata.permissions();
                    if perms.readonly() {
                        print!("remove readonly flag ... ");
                        #[allow(clippy::permissions_set_readonly_false)]
                        perms.set_readonly(false);
                        if let Err(e) = fs::set_permissions(f, perms) {
                            print!("Error: {:?} ... ", e);
                        }
                    }
                }
            }
            match fs::remove_file(f) {
                Ok(_) => {
                    println!("OK");
                    stats.files_deleted += 1;
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    stats.files_delete_failed += 1;
                }
            }
        }
    }

    fn remove_directories(destination: &ScanResult, stats: &mut Stats) {
        let mut dirs_to_remove = destination
            .data
            .iter()
            .filter(|(_k, d)| d.is_directory() && d.get_action() == PlannedAction::Delete)
            .map(|(_k, d)| d.get_full_path())
            .collect::<Vec<_>>();

        dirs_to_remove.sort_by(|a, b| b.as_os_str().len().cmp(&a.as_os_str().len()));

        for d in dirs_to_remove {
            print!("Remove: {:?} ... ", d);
            match fs::remove_dir(d) {
                Ok(_) => {
                    println!("OK");
                    stats.dirs_deleted += 1;
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    stats.dirs_delete_failed += 1;
                }
            }
        }
    }

    fn create_directories(source: &ScanResult, base: &PathBuf, stats: &mut Stats) -> Result<()> {
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

        for d in dirs_to_create.iter() {
            println!("Create directory: {:?}", d);
            fs::create_dir(d).with_context(|| format!("Cannot create directory {:?}", d))?;
            stats.dirs_created += 1;
        }

        Ok(())
    }

    fn copy_files(source: &ScanResult, base: &PathBuf, stats: &mut Stats) {
        let files_to_copy = source
            .data
            .iter()
            .filter(|(_k, f)| f.is_file() && f.get_action() == PlannedAction::Copy)
            .collect::<Vec<_>>();

        let total = files_to_copy.len();
        for (counter, (k, f)) in files_to_copy.into_iter().enumerate() {
            let copy_from = f.get_full_path();
            let mut copy_to = base.clone();
            copy_to.push(k);
            print!("{}/{} Copy {:?} to {:?}", counter + 1, total, copy_from, copy_to);
            match fs::copy(copy_from, copy_to) {
                Ok(_) => {
                    println!(" ... OK");
                    stats.files_copied += 1;
                }
                Err(e) => {
                    println!(" ... Error: {:?}", e);
                    stats.files_copy_failed += 1;
                }
            }
        }
    }

    /// Runs the planned actions. Single files or directories that fail are counted in
    /// the returned `Stats`; failing to create a directory stops the run with an error.
    pub fn execute(
        source: &ScanResult,
        destination: &ScanResult,
        arguments: &Arguments,
    ) -> Result<Stats> {
        let mut stats = Stats::default();
        Self::remove_files(destination, arguments, &mut stats);
        Self::remove_directories(destination, &mut stats);
        Self::create_directories(source, &destination.base, &mut stats)?;
        Self::copy_files(source, &destination.base, &mut stats);
        Ok(stats)
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

    fn sync(src: &str, dst: &str, remove_ro: bool) -> Result<Stats> {
        let args = Arguments {
            source: src.to_string(),
            destination: dst.to_string(),
            verbose: false,
            debug: false,
            remove_ro,
            report: None,
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

        fn sync(&self, remove_ro: bool) -> Result<Stats> {
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

    #[test]
    fn stats_count_what_was_done() {
        let f = Fixture::new("stats");
        f.t.file(r"src\a.txt", "a");
        f.t.file(r"src\sub\b.txt", "b");
        f.t.file(r"dst\old\x.txt", "x");
        f.t.file(r"dst\y.txt", "y");

        let stats = f.sync(false).unwrap();
        assert_eq!(
            stats,
            Stats {
                files_deleted: 2,
                dirs_deleted: 1,
                dirs_created: 1,
                files_copied: 2,
                ..Default::default()
            }
        );

        assert_eq!(f.sync(false).unwrap(), Stats::default());
    }

    #[test]
    fn stats_count_failures() {
        // `^cache$` excludes the directory but not the file inside it, so the
        // directory can't be created (copy fails) or deleted (it isn't empty).
        let f = Fixture::new("stats_failures");
        f.t.file(r"src\.backup_rules", "-: ^cache$");
        f.t.file(r"src\cache\c.txt", "c");

        let stats = f.sync(false).unwrap();
        assert_eq!(stats.files_copy_failed, 1);

        f.t.file(r"dst\cache\c.txt", "c");
        let modified = fs::metadata(f.src.join(r"cache\c.txt")).unwrap().modified().unwrap();
        File::options()
            .write(true)
            .open(f.dst.join(r"cache\c.txt"))
            .unwrap()
            .set_modified(modified)
            .unwrap();
        let stats = f.sync(false).unwrap();
        assert_eq!(stats.dirs_delete_failed, 1);
        assert_eq!(stats.errors(), 1);
    }
}
