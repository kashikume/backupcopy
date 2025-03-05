use anyhow::Result;
use std::env;

#[derive(Debug)]
pub struct Arguments {
    pub source: String,
    pub destination: String,
    pub verbose: bool,
    pub debug: bool,
    pub remove_ro: bool,
}

impl Arguments {
    pub fn parse() -> Result<Arguments> {
        let mut verbose = false;
        let mut debug = false;
        let mut remove_ro = false;
        let mut source: Option<String> = None;
        let mut destination: Option<String> = None;
        let mut app_name: Option<String> = None;

        for arg in env::args() {
            match arg.as_str() {
                "-v" | "--verbose" => verbose = true,
                "--debug" => debug = true,
                "--remove-ro" => remove_ro = true,
                _ => {
                    if app_name.is_none() {
                        app_name = Some(arg.clone());
                    } else if source.is_none() {
                        source = Some(arg.clone());
                    } else if destination.is_none() {
                        destination = Some(arg.clone());
                    } else {
                        return Err(anyhow::anyhow!("Unknown argument: {}", arg));
                    }
                }
            }
        }

        if source.is_none() || destination.is_none() {
            return Err(anyhow::anyhow!("Source and destination must be provided"));
        }

        Ok(Arguments {
            source: source.unwrap(),
            destination: destination.unwrap(),
            verbose,
            debug,
            remove_ro,
        })
    }

    pub fn print_usage() {
        println!("Usage: backupcopy [-v] [--debug] [--remove-ro] <source> <destination>");
    }
}
