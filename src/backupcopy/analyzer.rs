use super::fsitem::PlannedAction;
use super::{scanresult::ScanResult, scanrules::ScanRules};

pub struct Analyzer {}

impl Analyzer {
    pub fn plan_actions(source: &mut ScanResult, destination: &mut ScanResult, rules: &ScanRules) {
        for (k, d) in destination.data.iter_mut() {
            if source.data.contains_key(k) {
                let s = &source.data[k];
                if s.is_file() && d.is_directory() {
                    d.set_action(PlannedAction::Delete);
                } else if s.is_file() && d.is_file() {
                    if s.get_date() > d.get_date() || s.get_size() != d.get_size() {
                        d.set_action(PlannedAction::Delete);
                    }
                } else {
                    d.set_action(PlannedAction::Ignore);
                }
            } else {
                d.set_action(PlannedAction::Delete);
            }
        }
    }
}
