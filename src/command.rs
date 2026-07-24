use std::collections::{BTreeSet, VecDeque};
use std::error::Error;

use crate::parser::maybe_quote;

/// A block, consisting of multiple commands.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// The commands in the block.
    pub commands: Vec<Command>,

    /// The literal string of the input commands. Used to generate the output.
    pub(crate) literal: String,
    /// The block's line number position in the script.
    pub(crate) line_number: u32,
}

/// A command.
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    /// The name of the command. Never empty.
    pub name: String,
    /// The command's arguments, in the given order.
    pub args: Vec<Argument>,
    /// The command prefix, if given.
    pub prefix: Option<String>,
    /// Any command tags, if given.
    pub tags: BTreeSet<String>,
    /// Silences the output of this command. This is handled automatically, the
    /// [`Runner`](crate::Runner) does not have to take this into account.
    pub silent: bool,
    /// If true, the command is expected to fail with a panic or error. If the
    /// command does not fail, the test fails.
    pub fail: bool,

    /// The command's line number position in the script.
    pub(crate) line_number: u32,
}

impl Command {
    /// Returns an argument consumer, for more convenient argument processing.
    /// Does not affect [`Command::args`].
    ///
    /// See the [module documentation](crate#argument-processing) for usage
    /// examples.
    pub fn consume_args(&self) -> ArgumentConsumer<'_> {
        ArgumentConsumer::new(&self.args)
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&maybe_quote(&self.name))?;
        for arg in &self.args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}

/// A command argument.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Argument {
    /// A positional argument value. Can be empty.
    Positional(String),
    /// A `key=value` style argument, with the key followed by the value. Keys
    /// are not guaranteed to be unique; the [`Runner`](crate::Runner) can
    /// handle this as desired. Both the key and value can be empty.
    KeyValue(String, String),
}

impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Positional(value) => f.write_str(&maybe_quote(value)),
            Self::KeyValue(key, value) => {
                f.write_str(&maybe_quote(key))?;
                f.write_str("=")?;
                f.write_str(&maybe_quote(value))
            }
        }
    }
}

/// Helper for argument processing. Returns and removes arguments as requested.
///
/// Created by [`Command::consume_args()`]. Implements [`Iterator`], but is also
/// intended for out-of-order processing, unlike most iterators.
pub struct ArgumentConsumer<'a> {
    args: VecDeque<&'a Argument>,
}

impl<'a> Iterator for ArgumentConsumer<'a> {
    type Item = &'a Argument;

    fn next(&mut self) -> Option<Self::Item> {
        self.args.pop_front()
    }
}

impl<'a> ArgumentConsumer<'a> {
    /// Creates a new argument consumer.
    fn new(args: &'a [Argument]) -> Self {
        Self { args: VecDeque::from_iter(args.iter()) }
    }

    /// Returns and removes the next `KeyValue` argument, if any.
    pub fn next_key(&mut self) -> Option<(&'a str, &'a str)> {
        let index = self.args.iter().position(|arg| matches!(arg, Argument::KeyValue(_, _)))?;
        let Argument::KeyValue(key, value) = self.args.remove(index)? else { unreachable!() };
        Some((key, value))
    }

    /// Returns and removes the next `Positional` argument, if any.
    pub fn next_pos(&mut self) -> Option<&'a str> {
        let index = self.args.iter().position(|arg| matches!(arg, Argument::Positional(_)))?;
        let Argument::Positional(value) = self.args.remove(index)? else { unreachable!() };
        Some(value)
    }

    /// Looks up a `KeyValue` argument by key, removing it and returning its value.
    /// If multiple arguments have the same key, they are all removed and the last
    /// value is returned.
    pub fn take_key(&mut self, key: &str) -> Option<&'a str> {
        let Some(Argument::KeyValue(key, value)) =
            self.args.iter().rev().find(|arg| matches!(arg, Argument::KeyValue(k, _) if k == key))
        else {
            return None;
        };
        self.args.retain(|arg| !matches!(arg, Argument::KeyValue(k, _) if k == key));
        Some(value)
    }

    /// Rejects any remaining arguments with an error.
    pub fn reject_next(&self) -> Result<(), Box<dyn Error>> {
        if let Some(arg) = self.args.front() {
            return Err(format!("unexpected argument '{arg}'").into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructs an Argument from a string value or key => value.
    macro_rules! arg {
        ($value:expr) => {
            Argument::Positional($value.to_string())
        };
        ($key:expr => $value:expr) => {
            Argument::KeyValue($key.to_string(), $value.to_string())
        };
    }

    /// Constructs a Command by parsing the given input string.
    macro_rules! cmd {
        ($input:expr) => {{
            crate::parser::parse_command(&format!("{}\n", $input)).expect("invalid command")
        }};
    }

    /// Tests Command.consume_args(). ArgumentConsumer is tested separately.
    #[test]
    fn command_consume_args() {
        let cmd = cmd!("cmd foo key=value bar");
        assert!(cmd.consume_args().eq(&cmd.args));
    }

    /// Tests Command and Argument display formatting.
    #[test]
    fn display() {
        assert_eq!(arg!("value").to_string(), "value");
        assert_eq!(arg!("key" => "value").to_string(), "key=value");
        assert_eq!(
            arg!("key with spaces" => "line\nbreak").to_string(),
            r#""key with spaces"="line\nbreak""#
        );
        assert_eq!(arg!(r#"value "quoted""#).to_string(), r#""value \"quoted\"""#);

        assert_eq!(cmd!("command arg key=value").to_string(), "command arg key=value");
        assert_eq!(
            cmd!(r#""command with spaces" "arg with spaces" key="line\nbreak""#).to_string(),
            r#""command with spaces" "arg with spaces" key="line\nbreak""#
        );
    }

    /// Tests ArgumentConsumer.take_key().
    #[test]
    fn argument_consumer_take_key() {
        let cmd = cmd!("cmd value key=1 foo=bar key=2");

        // take_key() returns None on unknown keys, including ones that match a
        // positional argument.
        let mut args = cmd.consume_args();
        assert_eq!(args.take_key("unknown"), None);
        assert_eq!(args.take_key("value"), None);
        assert!(args.eq(&cmd.args));

        // take_key() removes duplicate keys, returning the last value.
        let mut args = cmd.consume_args();
        assert_eq!(args.take_key("key"), Some("2"));
        assert!(args.eq([&cmd.args[0], &cmd.args[2]]));

        // take_key() removes a single key and returns its value.
        let mut args = cmd.consume_args();
        assert_eq!(args.take_key("foo"), Some("bar"));
        assert!(args.eq([&cmd.args[0], &cmd.args[1], &cmd.args[3]]));
    }

    /// Tests ArgumentConsumer.next().
    #[test]
    fn argument_consumer_next() {
        let cmd = cmd!("cmd foo key=1 key=2 bar");

        let mut args = cmd.consume_args();
        assert_eq!(args.next(), Some(&cmd.args[0]));
        assert_eq!(args.next(), Some(&cmd.args[1]));
        assert_eq!(args.next(), Some(&cmd.args[2]));
        assert_eq!(args.next(), Some(&cmd.args[3]));
        assert_eq!(args.next(), None);
        assert_eq!(args.next_key(), None);
        assert_eq!(args.next_pos(), None);
    }

    /// Tests ArgumentConsumer.next_key().
    #[test]
    fn argument_consumer_next_key() {
        let cmd = cmd!("cmd foo key=1 key=2 bar");
        let mut args = cmd.consume_args();

        assert_eq!(args.next_key(), Some(("key", "1")));
        assert_eq!(args.next_key(), Some(("key", "2")));
        assert_eq!(args.next_key(), None);
        assert_eq!(args.next(), Some(&cmd.args[0]));
        assert_eq!(args.next(), Some(&cmd.args[3]));
        assert_eq!(args.next(), None);
    }

    /// Tests ArgumentConsumer.next_pos().
    #[test]
    fn argument_consumer_next_pos() {
        let cmd = cmd!("cmd foo key=1 key=2 bar");
        let mut args = cmd.consume_args();

        assert_eq!(args.next_pos(), Some("foo"));
        assert_eq!(args.next_pos(), Some("bar"));
        assert_eq!(args.next_pos(), None);
        assert_eq!(args.next(), Some(&cmd.args[1]));
        assert_eq!(args.next(), Some(&cmd.args[2]));
        assert_eq!(args.next(), None);
    }

    /// Tests ArgumentConsumer.reject_next().
    #[test]
    fn argument_consumer_reject_next() {
        // Empty args return Ok.
        let cmd = cmd!("cmd");
        assert!(cmd.consume_args().reject_next().is_ok());

        // Positional argument fails. It does not consume the arg.
        let cmd = cmd!("cmd value");
        let mut args = cmd.consume_args();
        assert_eq!(args.reject_next().unwrap_err().to_string(), "unexpected argument 'value'");
        assert!(args.next().is_some());

        // Key/value argument fails. It does not consume the arg.
        let cmd = cmd!("cmd key=value");
        let mut args = cmd.consume_args();
        assert_eq!(args.reject_next().unwrap_err().to_string(), "unexpected argument 'key=value'");
        assert!(args.next().is_some());
    }
}
