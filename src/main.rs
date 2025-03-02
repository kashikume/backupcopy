mod backupcopy;
use anyhow::Result;
use backupcopy::{
    analyzer::Analyzer, arguments::Arguments, executor::Executor, fsscanner::FsScanner,
};

fn run(args: &Arguments) -> Result<()> {
    let (mut source, rules) = FsScanner::scan(&args.source)?;
    let (mut destination, _) = FsScanner::scan(&args.destination)?;
    Analyzer::plan_actions(&mut source, &mut destination, &rules);
    Executor::execute(&source, &destination)?;
    Ok(())
}

fn main() {
    let args = match Arguments::parse() {
        Ok(args) => args,
        Err(e) => {
            println!("Error: {}", e);
            Arguments::print_usage();
            return;
        }
    };

    match run(&args) {
        Err(e) => println!("Error: {:?}", e),
        Ok(_) => println!("All done. Thank you."),
    }
}
