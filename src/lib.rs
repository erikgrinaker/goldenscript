//! This crate provides the Goldenscript testing framework, loosely based on
//! Cockroach Labs' [`datadriven`](https://github.com/cockroachdb/datadriven)
//! framework for Go. It combines several testing techniques that make it easy
//! and efficient to write and update test cases:
//!
//! * [Golden master testing](https://en.wikipedia.org/wiki/Characterization_test)
//!   (aka characterization testing or historical oracle)
//! * [Data-driven testing](https://en.wikipedia.org/wiki/Data-driven_testing)
//!   (aka table-driven testing or parameterized testing)
//! * [Keyword-driven testing](https://en.wikipedia.org/wiki/Keyword-driven_testing)
//!
//! A goldenscript is a plain text file that contains a set of arbitrary input
//! commands and their expected text output, separated by `---`:
//!
//! ```text
//! command
//! ---
//! output
//!
//! command argument key=value
//! ---
//! output
//! ```
//!
//! The commands are executed by a provided [`Runner`]. The expected output is
//! usually not written by hand, but instead generated by running tests with the
//! environment variable `UPDATE_GOLDENFILES=1` and then verified by inspection
//! before it is checked in to version control. Tests will fail with a diff if
//! they don't match the expected output.
//!
//! This approach is particularly useful when testing complex stateful systems,
//! such as computer language parsing, operations on a key/value store,
//! concurrent transactions in a SQL database, or communication between a
//! cluster of Raft nodes. It can be very tedious and labor-intensive to write
//! and assert such cases by hand, so scripting and recording these interactions
//! often yields much better test coverage at a fraction of the cost.
//!
//! Internally, the
//! [`goldenfile`](https://docs.rs/goldenfile/latest/goldenfile/) crate is used
//! to manage golden files.
//!
//! # Example
//!
//! We'll test the [`dateparser`](https://docs.rs/dateparser/latest/dateparser/)
//! crate which parses timestamp strings in various formats.
//!
//! We write an initial goldenscript `tests/scripts/dateparser` containing test
//! cases for a `parse` command. A goldenscript may contain multiple commands,
//! either as individual input/output blocks or grouped together (which will
//! append their output). The input ends with a `---` separator, and the output
//! ends with a blank line. Note that we don't yet specify any expected output
//! after the `---` separator, this will be autogenerated later.
//!
//! ```text
//! parse 2024-04-30
//! ---
//!
//! # Test various date formats.
//! parse 2024-Apr-30
//! parse 2024.04.30
//! parse 04/30/2024
//! ---
//!
//! # Test some error cases.
//! parse 30.04.2024
//! parse 30/04/2024
//! parse 30/04/24
//! ---
//!
//! # Strings containing special characters must be quoted using " or '.
//! parse "2024-04-30 11:55:32"
//! parse '2024年04月30日11时55分32秒'
//! ---
//! ```
//!
//! We write a runner that recognizes this `parse` command and its timestamp
//! argument, and outputs the parsed date/time in RFC 3339 format. We also add a
//! test using it to run the goldenscript `tests/scripts/dateparser`.
//!
//! ```no_run
//! struct DateParserRunner;
//!
//! impl goldenscript::Runner for DateParserRunner {
//!     fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn std::error::Error>> {
//!         // Only accept a parse command with a single argument.
//!         if command.name != "parse" {
//!             return Err(format!("invalid command {}", command.name).into())
//!         }
//!         if command.args.len() != 1 {
//!             return Err("parse takes 1 argument".into())
//!         }
//!
//!         // Parse the timestamp, and output the RFC 3339 timestamp or error string.
//!         let input = &command.args[0].value;
//!         match dateparser::parse_with(input, &chrono::offset::Utc, chrono::NaiveTime::MIN) {
//!             Ok(datetime) => Ok(datetime.to_rfc3339()),
//!             Err(error) => Ok(format!("Error: {error}")),
//!         }
//!     }
//! }
//!
//! #[test]
//! fn dateparser() -> std::io::Result<()> {
//!     goldenscript::run(&mut DateParserRunner, "tests/scripts/dateparser")
//! }
//! ```
//!
//! Running `UPDATE_GOLDENFILES=1 cargo test` will populate the file with the
//! runner's output. We verify by inspection that it is correct. Later runs of
//! `cargo test` will assert that the output matches the file.
//!
//! ```text
//! parse 2024-04-30
//! ---
//! 2024-04-30T00:00:00+00:00
//!
//! # Test various other date formats.
//! parse 2024-Apr-30
//! parse 2024.04.30
//! parse 04/30/2024
//! ---
//! 2024-04-30T00:00:00+00:00
//! 2024-04-30T00:00:00+00:00
//! 2024-04-30T00:00:00+00:00
//!
//! # Test some error cases.
//! parse 30.04.2024
//! parse 30/04/2024
//! parse 30/04/24
//! ---
//! Error: 30.04.2024 did not match any formats.
//! Error: 30/04/2024 did not match any formats.
//! Error: 30/04/24 did not match any formats.
//!
//! # Strings containing special characters must be quoted using " or '.
//! parse "2024-04-30 11:55:32"
//! parse '2024年04月30日11时55分32秒'
//! ---
//! 2024-04-30T11:55:32+00:00
//! 2024-04-30T11:55:32+00:00
//! ```
//!
//! # Syntax
//!
//! ## Blocks
//!
//! A goldenscript consists of one or more input/output blocks. Each block has a
//! set of one or more input commands on individual lines (empty or comment
//! lines are ignored), a `---` separator, and arbitrary output terminated by an
//! empty line. A minimal goldenscript with two blocks might be:
//!
//! ```text
//! command
//! ---
//! output
//!
//! command 1
//! command 2
//! ---
//! output 1
//! output 2
//! ```
//!
//! ## Commands
//!
//! A [`Command`] must have a command name, which can be any arbitrary non-empty
//! [string](#strings), e.g.:
//!
//! ```text
//! command
//! "command with space and 🚀"
//! ---
//! ```
//!
//! It may additionally have:
//!
//! * [**Arguments:**](Argument) any number of space-separated arguments.
//!   These have a string [value](Argument::value), and optionally also a string
//!   [key](Argument::key) as `key=value`. Values can be empty, and duplicate
//!   keys are allowed by the parser (the runner can handle this as desired).
//!
//!     ```text
//!     command argument key=value
//!     command "argument with space" "key with space"="value with space"
//!     command "" key=  # Empty argument values.
//!     ---
//!     ```
//!
//! * [**Prefix:**](Command::prefix) an optional :-terminated string prefix
//!   before the command. The command's output will be given the same prefix.
//!   The prefix can be used by the test runner, e.g. to signify two different
//!   clients.
//!
//!     ```text
//!     client1: put key=value
//!     client2: get key
//!     ---
//!     client1: put ok
//!     client2: get key=value
//!     ```
//!
//! * [**Silencing:**](Command::silent) a command wrapped in `()` will have its
//!   output suppressed. This can be useful e.g. for setup commands whose output
//!   are not of interest in the current test case and would only add noise.
//!
//!     ```text
//!     echo foo
//!     (echo bar)
//!     ---
//!     foo
//!     ```
//!
//! * [**Failure:**](Command::fail) if `!` precedes the command, it is expected
//!   to fail with an error or panic, and the failure message is used as output.
//!   If the command unexpectedly succeeds, the test fails. If the line contains
//!   other symbols before the command name (e.g. a prefix or silencing), the
//!   `!` must be used immediately before the command name.
//!
//!     ```text
//!     ! command error=foo
//!     prefix: ! command panic=bar
//!     (!command error=foo)
//!     ---
//!     Error: foo
//!     prefix: Panic: bar
//!     ```
//!
//! ## Output
//!
//! The command output following a `---` separator can contain any arbitrary
//! Unicode string until an empty line (or end of file). If the command output
//! contains empty lines, the entire output will automatically be prefixed with
//! `> `. If no commands in a block yield any output, it defaults to "ok".
//!
//! ```text
//! echo "output 1"
//! echo "output 2"
//! ---
//! output 1
//! output 2
//!
//! echo "Paragraph 1.\n\nParagraph 2."
//! ---
//! > Paragraph 1.
//! >
//! > Paragraph 2.
//!
//! echo "输出\n# Comment\n🚀"
//! ---
//! 输出
//! # Comment
//! 🚀
//! ```
//!
//! ## Comments
//!
//! Comments begin with `#` or `//` and run to the end of the line.
//!
//! ```text
//! # This is a comment.
//! // As is this.
//! command argument # Comments can follow commands too.
//! ---
//! ```
//!
//! ## Strings
//!
//! Unquoted strings can only contain alphanumeric ASCII characters
//! `[a-zA-Z0-9]` and a handful of special characters: `_ - . / @`
//! (only `_` at the start of a string).
//!
//! Strings can be quoted using `"` or `'`, in which case they can contain
//! arbitrary Unicode characters. `\` is used as an escape character, both to
//! escape quotes `\"` and `\'` as well as itself `\\`, and also `\0` (null),
//! `\n` (newline), `\r` (carriage return), and `\t` (tab).
//!
//! ```text
//! string
//! "string with spaces and \"quotes\""
//! '字符串'
//! ---
//! ```
//!
//! # Writing Tests
//!
//! In the simplest case, a goldenscript test might be:
//!
//! ```no_run
//! # use std::error::Error;
//! struct Runner;
//!
//! impl goldenscript::Runner for Runner {
//!     fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
//!         match command.name.as_str() {
//!             "echo" => {
//!                 let lines: Vec<&str> = command.args.iter().map(|a| a.value.as_str()).collect();
//!                 Ok(lines.join("\n"))
//!             }
//!             name => return Err(format!("invalid command {name}").into())
//!         }
//!     }
//! }
//!
//! #[test]
//! fn test() -> std::io::Result<()> {
//!     goldenscript::run(&mut Runner, "tests/scripts/test")
//! }
//! ```
//!
//! ## Argument Processing
//!
//! Arguments can be processed manually via [`Command::args`], or using the
//! [`Command::consume_args()`] helper which simplifies common argument
//! handling. For example:
//!
//! ```
//! # use std::error::Error;
//! # struct Runner;
//! # impl Runner {
//! #   fn send(&self, ids: &[u32], message: &str, retry: bool) -> Result<String, Box<dyn Error>> {
//! #     Ok(String::new())
//! #   }
//! # }
//! #
//! impl goldenscript::Runner for Runner {
//!     /// Implement a send command, which sends a string message to a list
//!     /// of nodes, optionally retrying.
//!     ///
//!     /// send [retry=BOOL] MESSAGE ID...
//!     ///
//!     /// Example: send foo 1 2 3
//!     fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
//!         if command.name != "send" {
//!             return Err(format!("invalid command {}", command.name).into())
//!         }
//!
//!         let mut args = command.consume_args();
//!
//!         // The first positional argument is a required string message.
//!         let message = &args.next_pos().ok_or("message not given")?.value;
//!
//!         // The remaining positional arguments are numeric node IDs.
//!         let ids: Vec<u32> = args.rest_pos().iter().map(|a| a.parse()).collect::<Result<_, _>>()?;
//!         if ids.is_empty() {
//!             return Err("no node IDs given".into())
//!         }
//!
//!         // An optional retry=bool key/value argument can also be given.
//!         let retry: bool = args.lookup_parse("retry")?.unwrap_or(false);
//!
//!         // Any other arguments that haven't been processed above should error.
//!         args.reject_rest()?;
//!
//!         // Execute the send.
//!         self.send(&ids, message, retry)
//!     }
//! }
//! ```
//!
//! ## Managing State
//!
//! The runner is free to manage internal state as desired. If it is stateful,
//! it is recommended to persist state within a single goldenscript (across
//! commands and blocks), but not across goldenscripts since this can be hard to
//! reason about and depend on the execution order of scripts. This is most
//! easily done by instantiating a new runner for each script.
//!
//! Initial state setup should generally be done via explicit setup commands, to
//! make it more discoverable.
//!
//! ## Running All Scripts in a Directory
//!
//! External crates can be used to automatically generate and run individual
//! tests for each goldenscript in a directory. For example, the
//! [`test_each_file`](https://docs.rs/test_each_file/latest/test_each_file/)
//! crate:
//!
//! ```no_run
//! # use std::error::Error;
//! # struct Runner;
//! #
//! # impl goldenscript::Runner for Runner {
//! #     fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> { todo!() }
//! # }
//! use test_each_file::test_each_path;
//!
//! test_each_path! { in "tests/scripts" as scripts => test_goldenscript }
//!
//! fn test_goldenscript(path: &std::path::Path) {
//!     goldenscript::run(&mut Runner, path).unwrap()
//! }
//! ```
//!
//! ## Hooks
//!
//! Runners have various hooks that will be called during script execution:
//! [`Runner::start_script`], [`Runner::end_script`], [`Runner::start_block`],
//! [`Runner::end_block`], [`Runner::start_command`], and
//! [`Runner::end_command`]. These can be used e.g. for initial setup, invariant
//! assertions, or to output the current state.

#![warn(clippy::all)]

mod command;
mod parser;
mod runner;

pub use command::{Argument, ArgumentConsumer, Command};
pub use runner::{generate, run, Runner};
