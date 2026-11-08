use super::fsitem::PlannedAction;
use super::scanrules::RuleAction;
use super::{scanresult::ScanResult, scanrules::ScanRules};

pub struct Analyzer {}

impl Analyzer {
    pub fn plan_actions(source: &mut ScanResult, destination: &mut ScanResult, rules: &ScanRules) {
        for (k, d) in destination.data.iter_mut() {
            if source.data.contains_key(k) {
                let s = &source.data[k];
                if (s.is_file() && d.is_directory()) || (s.is_directory() && d.is_file()) {
                    d.set_action(PlannedAction::Delete);
                } else if s.is_file() && d.is_file() {
                    if s.get_date() > d.get_date()
                        || s.get_size() != d.get_size()
                        || rules.evaluate(s.get_full_path()) == RuleAction::Remove
                    {
                        d.set_action(PlannedAction::Delete);
                    } else {
                        d.set_action(PlannedAction::Ignore);
                    }
                } else {
                    d.set_action(match rules.evaluate(s.get_full_path()) {
                        RuleAction::Keep => PlannedAction::Ignore,
                        RuleAction::Remove => PlannedAction::Delete,
                    });
                }
            } else {
                d.set_action(PlannedAction::Delete);
            }
        }

        for (k, s) in source.data.iter_mut() {
            if destination.data.contains_key(k)
                && destination.data[k].get_action() == PlannedAction::Ignore
            {
                s.set_action(PlannedAction::Ignore);
            } else {
                s.set_action(match rules.evaluate(s.get_full_path()) {
                    RuleAction::Keep => PlannedAction::Copy,
                    RuleAction::Remove => PlannedAction::Ignore,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    enum Item {
        Dir(&'static str),
        File(&'static str, u64, u64),
    }
    use Item::*;

    fn scan(base: &str, items: &[Item]) -> ScanResult {
        let mut r = ScanResult::new(PathBuf::from(base));
        for item in items {
            match item {
                Dir(p) => r.add_dir(PathBuf::from(base).join(p)),
                File(p, size, date) => r.add_file(PathBuf::from(base).join(p), *size, *date),
            }
        }
        r
    }

    fn rules(content: &str) -> ScanRules {
        let mut r = ScanRules::new();
        r.add_rules_from_str(&PathBuf::from(r"C:\src\.backup_rules"), content)
            .unwrap();
        r
    }

    fn action(r: &ScanResult, rel: &str) -> PlannedAction {
        r.data[&PathBuf::from(rel)].get_action()
    }

    fn plan(src: &[Item], dst: &[Item], rules_text: &str) -> (ScanResult, ScanResult) {
        let mut s = scan(r"C:\src", src);
        let mut d = scan(r"D:\dst", dst);
        Analyzer::plan_actions(&mut s, &mut d, &rules(rules_text));
        (s, d)
    }

    #[test]
    fn new_source_items_are_copied() {
        let (s, _) = plan(&[Dir("sub"), File(r"sub\a.txt", 1, 1)], &[], "");
        assert_eq!(action(&s, "sub"), PlannedAction::Copy);
        assert_eq!(action(&s, r"sub\a.txt"), PlannedAction::Copy);
    }

    #[test]
    fn destination_items_missing_in_source_are_deleted() {
        let (_, d) = plan(&[], &[Dir("old"), File(r"old\a.txt", 1, 1)], "");
        assert_eq!(action(&d, "old"), PlannedAction::Delete);
        assert_eq!(action(&d, r"old\a.txt"), PlannedAction::Delete);
    }

    #[test]
    fn unchanged_file_is_ignored() {
        let (s, d) = plan(&[File("a.txt", 5, 100)], &[File("a.txt", 5, 100)], "");
        assert_eq!(action(&s, "a.txt"), PlannedAction::Ignore);
        assert_eq!(action(&d, "a.txt"), PlannedAction::Ignore);
    }

    #[test]
    fn newer_source_file_replaces_destination() {
        let (s, d) = plan(&[File("a.txt", 5, 200)], &[File("a.txt", 5, 100)], "");
        assert_eq!(action(&d, "a.txt"), PlannedAction::Delete);
        assert_eq!(action(&s, "a.txt"), PlannedAction::Copy);
    }

    #[test]
    fn different_size_replaces_destination() {
        let (s, d) = plan(&[File("a.txt", 6, 100)], &[File("a.txt", 5, 100)], "");
        assert_eq!(action(&d, "a.txt"), PlannedAction::Delete);
        assert_eq!(action(&s, "a.txt"), PlannedAction::Copy);
    }

    #[test]
    fn newer_destination_with_same_size_is_kept() {
        let (s, d) = plan(&[File("a.txt", 5, 100)], &[File("a.txt", 5, 200)], "");
        assert_eq!(action(&d, "a.txt"), PlannedAction::Ignore);
        assert_eq!(action(&s, "a.txt"), PlannedAction::Ignore);
    }

    #[test]
    fn existing_directory_is_kept() {
        let (s, d) = plan(&[Dir("sub")], &[Dir("sub")], "");
        assert_eq!(action(&d, "sub"), PlannedAction::Ignore);
        assert_eq!(action(&s, "sub"), PlannedAction::Ignore);
    }

    #[test]
    fn file_replaced_by_directory() {
        let (s, d) = plan(&[Dir("x")], &[File("x", 1, 1)], "");
        assert_eq!(action(&d, "x"), PlannedAction::Delete);
        assert_eq!(action(&s, "x"), PlannedAction::Copy);
    }

    #[test]
    fn directory_replaced_by_file() {
        let (s, d) = plan(&[File("x", 1, 1)], &[Dir("x")], "");
        assert_eq!(action(&d, "x"), PlannedAction::Delete);
        assert_eq!(action(&s, "x"), PlannedAction::Copy);
    }

    #[test]
    fn excluded_new_items_are_not_copied() {
        let (s, _) = plan(
            &[Dir("build"), File(r"build\app.exe", 1, 1), File("a.tmp", 1, 1)],
            &[],
            "-: ^build(\\\\|$)\n-: \\.tmp$",
        );
        assert_eq!(action(&s, "build"), PlannedAction::Ignore);
        assert_eq!(action(&s, r"build\app.exe"), PlannedAction::Ignore);
        assert_eq!(action(&s, "a.tmp"), PlannedAction::Ignore);
    }

    #[test]
    fn excluded_items_already_in_destination_are_deleted() {
        let (s, d) = plan(
            &[Dir("build"), File(r"build\app.exe", 1, 1), File("a.tmp", 1, 1)],
            &[Dir("build"), File(r"build\app.exe", 1, 1), File("a.tmp", 1, 1)],
            "-: ^build(\\\\|$)\n-: \\.tmp$",
        );
        assert_eq!(action(&d, "build"), PlannedAction::Delete);
        assert_eq!(action(&d, r"build\app.exe"), PlannedAction::Delete);
        assert_eq!(action(&d, "a.tmp"), PlannedAction::Delete);
        assert_eq!(action(&s, "build"), PlannedAction::Ignore);
        assert_eq!(action(&s, r"build\app.exe"), PlannedAction::Ignore);
        assert_eq!(action(&s, "a.tmp"), PlannedAction::Ignore);
    }

    #[test]
    fn keep_rule_overrides_exclusion() {
        let (s, _) = plan(
            &[File("a.log", 1, 1), File("keep.log", 1, 1)],
            &[],
            "-: \\.log$\n+: ^keep\\.log$",
        );
        assert_eq!(action(&s, "a.log"), PlannedAction::Ignore);
        assert_eq!(action(&s, "keep.log"), PlannedAction::Copy);
    }
}
