use anyhow::Result;
use std::env;

#[derive(Debug)]
pub struct Arguments {
    pub source: String,
    pub destination: String,
    pub verbose: bool,
    pub debug: bool,
    pub remove_ro: bool,
    pub report: Option<String>,
}

impl Arguments {
    pub fn parse() -> Result<Arguments> {
        Self::parse_from(env::args())
    }

    /// Parses arguments from `args`; the first item is the program name.
    fn parse_from(args: impl IntoIterator<Item = String>) -> Result<Arguments> {
        let mut verbose = false;
        let mut debug = false;
        let mut remove_ro = false;
        let mut source: Option<String> = None;
        let mut destination: Option<String> = None;
        let mut app_name: Option<String> = None;
        let mut report: Option<String> = None;

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-v" | "--verbose" => verbose = true,
                "--debug" => debug = true,
                "--remove-ro" => remove_ro = true,
                "--report" => match args.next() {
                    Some(file) => report = Some(file),
                    None => return Err(anyhow::anyhow!("--report requires a file name")),
                },
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
            report,
        })
    }

    pub fn print_usage() {
        println!(
            "Usage: backupcopy [-v] [--debug] [--remove-ro] [--report <file>] <source> <destination>"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Arguments> {
        Arguments::parse_from(
            std::iter::once("backupcopy")
                .chain(args.iter().copied())
                .map(String::from),
        )
    }

    #[test]
    fn source_and_destination_without_flags() {
        let a = parse(&["src", "dst"]).unwrap();
        assert_eq!(a.source, "src");
        assert_eq!(a.destination, "dst");
        assert!(!a.verbose && !a.debug && !a.remove_ro);
        assert_eq!(a.report, None);
    }

    #[test]
    fn report_file() {
        let a = parse(&["--report", "log.txt", "src", "dst"]).unwrap();
        assert_eq!(a.report.as_deref(), Some("log.txt"));
        assert_eq!(a.source, "src");
        assert_eq!(a.destination, "dst");

        let a = parse(&["src", "dst", "--report", "log.txt"]).unwrap();
        assert_eq!(a.report.as_deref(), Some("log.txt"));
    }

    #[test]
    fn report_without_file_is_an_error() {
        let e = parse(&["src", "dst", "--report"]).unwrap_err();
        assert!(e.to_string().contains("--report"));
    }

    #[test]
    fn all_flags() {
        let a = parse(&["-v", "--debug", "--remove-ro", "src", "dst"]).unwrap();
        assert!(a.verbose && a.debug && a.remove_ro);
        assert_eq!(a.source, "src");
        assert_eq!(a.destination, "dst");
    }

    #[test]
    fn long_verbose_flag() {
        assert!(parse(&["--verbose", "src", "dst"]).unwrap().verbose);
    }

    #[test]
    fn flags_can_go_anywhere() {
        let a = parse(&["src", "--remove-ro", "dst", "-v"]).unwrap();
        assert!(a.verbose && a.remove_ro);
        assert_eq!(a.source, "src");
        assert_eq!(a.destination, "dst");
    }

    #[test]
    fn missing_destination_is_an_error() {
        assert!(parse(&["src"]).is_err());
    }

    #[test]
    fn no_paths_is_an_error() {
        assert!(parse(&["-v"]).is_err());
    }

    #[test]
    fn extra_argument_is_an_error() {
        let e = parse(&["src", "dst", "more"]).unwrap_err();
        assert!(e.to_string().contains("more"));
    }
}
