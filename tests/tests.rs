#![warn(clippy::all)]

use std::error::Error;
use std::io::Write as _;
use test_each_file::test_each_path;

// Run goldenscripts in tests/scripts that debug-print the commands.
test_each_path! { in "tests/scripts" as scripts => test_goldenscript }

fn test_goldenscript(path: &std::path::Path) {
    goldenscript::run(&mut DebugRunner::new(), path).expect("runner failed")
}

// Run goldenscripts in tests/generate with output in a separate file. This is
// particularly useful for parser tests where output hasn't yet been generated.
test_each_path! { for ["in", "out"] in "tests/generate" as generate => test_generate }

fn test_generate([in_path, out_path]: [&std::path::Path; 2]) {
    let input = std::fs::read_to_string(in_path).expect("failed to read file");
    let output = goldenscript::generate(&mut DebugRunner::new(), &input).expect("runner failed");

    let dir = out_path.parent().expect("invalid path");
    let filename = out_path.file_name().expect("invalid path");
    let mut mint = goldenfile::Mint::new(dir);
    let mut f = mint.new_goldenfile(filename).expect("failed to create goldenfile");
    f.write_all(output.as_bytes()).expect("failed to write output");
}

// Generate error tests for each pair of *.in and *.error files in tests/errors/.
// The input scripts are expected to error or panic with the stored output.
test_each_path! { for ["in", "error"] in "tests/errors" as errors => test_error }

fn test_error([in_path, out_path]: [&std::path::Path; 2]) {
    let input = std::fs::read_to_string(in_path).expect("failed to read file");
    let run =
        std::panic::AssertUnwindSafe(|| goldenscript::generate(&mut DebugRunner::new(), &input));
    let message = match std::panic::catch_unwind(run) {
        Ok(Ok(_)) => panic!("script succeeded"),
        Ok(Err(e)) => e.to_string(),
        Err(panic) => panic
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| std::panic::resume_unwind(panic)),
    };

    let dir = out_path.parent().expect("invalid path");
    let filename = out_path.file_name().expect("invalid path");
    let mut mint = goldenfile::Mint::new(dir);
    let mut f = mint.new_goldenfile(filename).expect("failed to create goldenfile");
    f.write_all(message.as_bytes()).expect("failed to write goldenfile");
}

/// A goldenscript runner that debug-prints the parsed command. It
/// understands the following special commands:
///
/// _echo: prints back the arguments, space-separated
/// _error: errors with the given string
/// _panic: panics with the given string
/// _set: sets various options
///
///   - prefix=<string>: printed immediately before the command output
///   - suffix=<string>: printed immediately after the command output
///   - start_block=<string>: printed at the start of a block
///   - start_command=<string>: printed at the start of a command
///   - end_block=<string>: printed at the end of a block
///   - end_command=<string>: printed at the end of a command
///
/// If a command is expected to fail via !, the parsed command string is
/// returned as an error.
#[derive(Default)]
struct DebugRunner {
    prefix: String,
    suffix: String,
    start_block: String,
    end_block: String,
    start_command: String,
    end_command: String,
}

impl DebugRunner {
    fn new() -> Self {
        Self::default()
    }
}

impl goldenscript::Runner for DebugRunner {
    fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
        // Process commands.
        let output = match command.name.as_str() {
            "_echo" => {
                for arg in &command.args {
                    if arg.key.is_some() {
                        return Err("echo args can't have keys".into());
                    }
                }
                command.args.iter().map(|a| a.value.clone()).collect::<Vec<String>>().join(" ")
            }

            "_error" => {
                let message = command.args.first().map(|a| a.value.as_str()).unwrap_or("error");
                return Err(message.to_string().into());
            }

            "_panic" => {
                let message = command.args.first().map(|a| a.value.as_str()).unwrap_or("panic");
                panic!("{message}");
            }

            "_set" => {
                for arg in &command.args {
                    match arg.key.as_deref() {
                        Some("prefix") => self.prefix = arg.value.clone(),
                        Some("suffix") => self.suffix = arg.value.clone(),
                        Some("start_block") => self.start_block = arg.value.clone(),
                        Some("end_block") => self.end_block = arg.value.clone(),
                        Some("start_command") => self.start_command = arg.value.clone(),
                        Some("end_command") => self.end_command = arg.value.clone(),
                        Some(key) => return Err(format!("unknown argument key {key}").into()),
                        None => return Err("argument must have a key".into()),
                    }
                }
                return Ok(String::new());
            }

            _ if command.fail => return Err(format!("{command:?}").into()),

            _ => format!("{command:?}"),
        };

        Ok(format!("{}{output}{}", self.prefix, self.suffix))
    }

    fn start_block(&mut self) -> Result<String, Box<dyn Error>> {
        Ok(self.start_block.clone())
    }

    fn end_block(&mut self) -> Result<String, Box<dyn Error>> {
        Ok(self.end_block.clone())
    }

    fn start_command(&mut self, _: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
        Ok(self.start_command.clone())
    }

    fn end_command(&mut self, _: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
        Ok(self.end_command.clone())
    }
}

/// A runner for BTreeMap tests. This is used as a documentation example.
#[derive(Default)]
struct BTreeMapRunner {
    map: std::collections::BTreeMap<String, String>,
}

impl goldenscript::Runner for BTreeMapRunner {
    fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        match command.name.as_str() {
            // get KEY: fetches the value of the given key, or None if it does not exist.
            "get" => {
                let mut args = command.consume_args();
                let key = &args.next_pos().ok_or("key not given")?.value;
                args.reject_rest()?;
                let value = self.map.get(key);
                output.push_str(&format!("get → {value:?}\n"))
            }

            // insert KEY=VALUE...: inserts the given key/value pairs, returning the old value.
            "insert" => {
                let mut args = command.consume_args();
                for arg in args.rest_key() {
                    let old = self.map.insert(arg.key.clone().unwrap(), arg.value.clone());
                    output.push_str(&format!("insert → {old:?}\n"));
                }
                args.reject_rest()?;
            }

            // range [FROM] [TO]: iterates over the key/value pairs in the range from..to.
            "range" => {
                use std::ops::Bound::*;
                let mut args = command.consume_args();
                let from = args.next_pos().map(|a| Included(a.value.clone())).unwrap_or(Unbounded);
                let to = args.next_pos().map(|a| Excluded(a.value.clone())).unwrap_or(Unbounded);
                args.reject_rest()?;
                for (key, value) in self.map.range((from, to)) {
                    output.push_str(&format!("{key}={value}\n"));
                }
            }

            name => return Err(format!("invalid command {name}").into()),
        };
        Ok(output)
    }
}

#[test]
fn btreemap() {
    goldenscript::run(&mut BTreeMapRunner::default(), "tests/btreemap")
        .expect("goldenscript failed")
}
