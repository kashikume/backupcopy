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
