#![warn(clippy::all)]

use std::io::Write as _;
use test_each_file::test_each_path;

// Run goldenscripts in tests/scripts that debug-print the commands.
test_each_path! { in "tests/scripts" as scripts => test_goldenscript }

fn test_goldenscript(path: &std::path::Path) {
    goldenscript::run(&mut DebugRunner::new(), path).expect("runner failed")
}

// Generate error tests for each pair of *.in and *.error files in tests/errors/.
// The input scripts are expected to error with the stored output.
test_each_path! { for ["in", "error"] in "tests/errors" as errors => test_error }

fn test_error([input, output]: [&std::path::Path; 2]) {
    let error = goldenscript::run(&mut DebugRunner::new(), input).expect_err("unexpected success");
    let dir = output.parent().expect("invalid path");
    let filename = output.file_name().expect("invalid path");
    let mut mint = goldenfile::Mint::new(dir);
    let mut f = mint.new_goldenfile(filename).expect("failed to create goldenfile");
    f.write_all(format!("{error}").as_bytes()).expect("failed to write goldenfile");
}

/// A goldenscript runner that debug-prints the parsed command. It
/// understands the following special commands:
///
/// _echo: prints back the arguments, one per line
/// _set: sets various options
///
///   - prefix=<string>: printed immediately before the command output
///   - suffix=<string>: printed immediately after the command output
///   - start_block=<string>: printed at the start of a block
///   - end_block=<string>: printed at the end of a block
///
#[derive(Default)]
struct DebugRunner {
    prefix: String,
    suffix: String,
    start_block: String,
    end_block: String,
}

impl DebugRunner {
    fn new() -> Self {
        Self::default()
    }
}

impl goldenscript::Runner for DebugRunner {
    fn run(&mut self, command: &goldenscript::Command) -> Result<String, String> {
        // Process commands.
        let output = match command.name.as_str() {
            "_echo" => {
                for arg in &command.args {
                    if arg.key.is_some() {
                        return Err("echo args can't have keys".to_string());
                    }
                }
                command.args.iter().map(|a| a.value.clone()).collect::<Vec<String>>().join(" ")
            }

            "_set" => {
                for arg in &command.args {
                    match arg.key.as_deref() {
                        Some("prefix") => self.prefix = arg.value.clone(),
                        Some("suffix") => self.suffix = arg.value.clone(),
                        Some("start_block") => self.start_block = arg.value.clone(),
                        Some("end_block") => self.end_block = arg.value.clone(),
                        Some(key) => return Err(format!("unknown argument key {key}")),
                        None => return Err("argument must have a key".to_string()),
                    }
                }
                return Ok(String::new());
            }

            _ => format!("{command:?}"),
        };

        Ok(format!("{}{output}{}", self.prefix, self.suffix))
    }

    fn start_block(&mut self) -> Result<String, String> {
        Ok(self.start_block.clone())
    }

    fn end_block(&mut self) -> Result<String, String> {
        Ok(self.end_block.clone())
    }
}

/// A runner for dateparser tests. This is used to generate example
/// goldenscripts for the documentation in src/lib.rs.
struct DateParserRunner;

impl goldenscript::Runner for DateParserRunner {
    fn run(&mut self, command: &goldenscript::Command) -> Result<String, String> {
        if command.name != "parse" {
            return Err(format!("invalid command {}", command.name));
        }
        if command.args.len() != 1 {
            return Err("parse takes 1 argument".to_string());
        }
        let input = &command.args[0].value;
        match ::dateparser::parse_with(input, &chrono::offset::Utc, chrono::NaiveTime::MIN) {
            Ok(datetime) => Ok(datetime.to_rfc3339()),
            Err(error) => Ok(format!("Error: {error}")),
        }
    }
}

// Run goldenscripts in tests/dateparser, for use in documentation.
test_each_path! { in "tests/dateparser" as dateparser => test_dateparser }

fn test_dateparser(path: &std::path::Path) {
    goldenscript::run(&mut DateParserRunner, path).expect("runner failed")
}
