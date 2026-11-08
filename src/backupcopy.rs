pub mod analyzer;
pub mod arguments;
pub mod executor;
pub mod fsitem;
pub mod fsscanner;
pub mod scanresult;
pub mod scanrules;

#[cfg(test)]
pub mod testutil {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A unique temporary directory that is deleted (read-only files included) when dropped.
    pub struct TestDir {
        pub path: PathBuf,
    }

    impl TestDir {
        pub fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "backupcopy_test_{}_{}_{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::SeqCst),
                name
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            TestDir { path }
        }

        pub fn dir(&self, rel: &str) -> PathBuf {
            let p = self.path.join(rel);
            fs::create_dir_all(&p).unwrap();
            p
        }

        pub fn file(&self, rel: &str, content: &str) -> PathBuf {
            let p = self.path.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, content).unwrap();
            p
        }

        pub fn path_string(&self, rel: &str) -> String {
            self.path.join(rel).to_str().unwrap().to_string()
        }
    }

    pub fn set_readonly(path: &Path, readonly: bool) {
        let mut perms = fs::metadata(path).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(readonly);
        fs::set_permissions(path, perms).unwrap();
    }

    fn clear_readonly_recursive(path: &Path) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    clear_readonly_recursive(&p);
                } else if fs::metadata(&p).map(|m| m.permissions().readonly()).unwrap_or(false) {
                    set_readonly(&p, false);
                }
            }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            clear_readonly_recursive(&self.path);
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
