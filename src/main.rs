mod backupcopy;
use anyhow::Result;
use backupcopy::{
    analyzer::Analyzer,
    arguments::Arguments,
    executor::{Executor, Stats},
    fsscanner::FsScanner,
    report,
};

fn run(args: &Arguments) -> Result<Stats> {
    if args.debug {
        println!("Arguments: {:?}", args);
    }
    if args.verbose {
        println!("Scanning Source...");
    }
    let (mut source, rules) = FsScanner::scan(&args.source)?;
    if args.verbose {
        println!("Scanning Destination...");
    }
    let (mut destination, _) = FsScanner::scan(&args.destination)?;

    if args.verbose {
        println!("Number of items in source: {}", source.data.len());
        println!("Number of items in destination: {}", destination.data.len());
    }

    Analyzer::plan_actions(&mut source, &mut destination, &rules);
    Executor::execute(&source, &destination, &args)
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

    let result = run(&args);
    match &result {
        Err(e) => println!("Error: {:?}", e),
        Ok(stats) => println!("All done ({}). Thank you.", stats.summary()),
    }

    if let Some(file) = &args.report {
        let line = report::line(
            &report::timestamp(),
            &args.source,
            &args.destination,
            &result,
        );
        if let Err(e) = report::append(file, &line) {
            println!("Error: {:?}", e);
        }
    }
}
