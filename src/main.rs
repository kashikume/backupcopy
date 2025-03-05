mod backupcopy;
use anyhow::Result;
use backupcopy::{
    analyzer::Analyzer, arguments::Arguments, executor::Executor, fsscanner::FsScanner,
};

fn run(args: &Arguments) -> Result<()> {
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
    Executor::execute(&source, &destination, &args)?;
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
