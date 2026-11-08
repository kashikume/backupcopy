use std::fs::OpenOptions;
use std::io::Write;

use anyhow::{Context, Error, Result};

use super::executor::Stats;

/// Current local time, formatted for the report.
pub fn timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Builds the report line for a run: `<timestamp> : SUCCESS|FAILURE <source> -> <destination>: (<details>)`.
pub fn line(timestamp: &str, source: &str, destination: &str, result: &Result<Stats>) -> String {
    let (status, details) = match result {
        Ok(stats) => ("SUCCESS", stats.summary()),
        Err(e) => ("FAILURE", single_line(e)),
    };
    format!(
        "{} : {} {} -> {}: ({})",
        timestamp, status, source, destination, details
    )
}

/// The error and its causes on one line, so that a run always adds exactly one line.
fn single_line(e: &Error) -> String {
    format!("{:#}", e)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Appends `line` to `file`, creating the file if it does not exist.
pub fn append(file: &str, line: &str) -> Result<()> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .with_context(|| format!("Cannot open report file {:?}", file))?;
    writeln!(f, "{}", line).with_context(|| format!("Cannot write report file {:?}", file))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backupcopy::testutil::TestDir;
    use anyhow::anyhow;
    use std::fs;

    #[test]
    fn success_line_without_errors() {
        let stats = Stats {
            files_copied: 3,
            dirs_created: 1,
            files_deleted: 2,
            dirs_deleted: 0,
            ..Default::default()
        };
        assert_eq!(
            line("2026-11-08 10:00:00", r"C:\src", r"D:\dst", &Ok(stats)),
            r"2026-11-08 10:00:00 : SUCCESS C:\src -> D:\dst: (copied 3 files, created 1 directory, deleted 2 files and 0 directories; no errors)"
        );
    }

    #[test]
    fn success_line_with_errors() {
        let stats = Stats {
            files_copied: 1,
            files_delete_failed: 5,
            dirs_delete_failed: 1,
            files_copy_failed: 2,
            ..Default::default()
        };
        assert_eq!(
            line("T", "s", "d", &Ok(stats)),
            "T : SUCCESS s -> d: (copied 1 file, created 0 directories, deleted 0 files and 0 directories; \
             8 errors: 5 files failed to delete in destination, \
             1 directory failed to delete in destination, 2 files failed to copy)"
        );
    }

    #[test]
    fn failure_line_is_a_single_line_with_causes() {
        let e = anyhow!("unclosed group\n  ^").context("Invalid rule\non line 2");
        assert_eq!(
            line("T", "s", "d", &Err(e)),
            "T : FAILURE s -> d: (Invalid rule on line 2: unclosed group ^)"
        );
    }

    #[test]
    fn append_adds_one_line_per_call() {
        let t = TestDir::new("report");
        let file = t.path_string("report.txt");
        append(&file, "first").unwrap();
        append(&file, "second").unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "first\nsecond\n");
    }

    #[test]
    fn append_to_missing_directory_is_an_error() {
        let t = TestDir::new("report_missing");
        assert!(append(&t.path_string(r"nope\report.txt"), "x").is_err());
    }
}
