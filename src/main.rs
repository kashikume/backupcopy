mod backupcopy;
use backupcopy::{analyzer::Analyzer, executor::Executor, fsscanner::FsScanner};

fn main() {
    let source_path = String::from("C:\\Users\\akola\\source");
    let dest_path = String::from("C:\\Users\\akola\\test_copy");
    let (mut source, rules) = FsScanner::scan(&source_path).unwrap();
    let (mut destination, _) = FsScanner::scan(&dest_path).unwrap();
    Analyzer::plan_actions(&mut source, &mut destination, &rules);
    // println!("source: {:?}", source);
    // println!("destination: {:?}", destination);
    Executor::execute(&source, &destination);
}
