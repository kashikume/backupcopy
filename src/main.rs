mod backupcopy;
use anyhow::Result;
use backupcopy::{analyzer::Analyzer, executor::Executor, fsscanner::FsScanner};

fn run() -> Result<()> {
    let source_path = String::from("C:\\Users\\akola\\source");
    let dest_path = String::from("C:\\Users\\akola\\test_copy");
    let (mut source, rules) = FsScanner::scan(&source_path).unwrap();
    let (mut destination, _) = FsScanner::scan(&dest_path).unwrap();
    Analyzer::plan_actions(&mut source, &mut destination, &rules);
    Executor::execute(&source, &destination)?;
    Ok(())
}

fn main() {
    match run() {
        Err(e) => println!("Error: {:?}", e),
        Ok(_) => println!("All done. Thank you."),
    }
}
