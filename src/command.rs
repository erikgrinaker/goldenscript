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

impl Argument {
    /// Returns the argument key, if this is a `key=value` style argument.
    pub fn key(&self) -> Option<&str> {
        match self {
            Self::Positional(_) => None,
            Self::KeyValue(key, _) => Some(key),
        }
    }

    /// Returns the argument value.
    pub fn value(&self) -> &str {
        match self {
            Self::Positional(value) | Self::KeyValue(_, value) => value,
        }
    }

    /// Parses the argument value as a T using core::str::parse(). Convenience
    /// method that returns an improved error message as a boxed error to ease
    /// error handling in a [`Runner`](crate::Runner).
    pub fn parse<T>(&self) -> Result<T, Box<dyn Error>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        self.value().parse().map_err(|e| format!("invalid argument '{}': {e}", self.value()).into())
    }
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

/// Helper for argument processing, by returning and removing arguments on
/// demand.
///
/// Created by [`Command::consume_args()`]. Implements [`Iterator`], but is also
/// intended for out-of-order processing, unlike most iterators.
pub struct ArgumentConsumer<'a> {
    args: VecDeque<&'a Argument>,
}

impl<'a> Iterator for ArgumentConsumer<'a> {
    type Item = &'a Argument;

    /// Returns and removes the next argument, if any.
    fn next(&mut self) -> Option<Self::Item> {
        self.args.pop_front()
    }
}

impl<'a> ArgumentConsumer<'a> {
    /// Creates a new argument consumer.
    fn new(args: &'a [Argument]) -> Self {
        Self { args: VecDeque::from_iter(args.iter()) }
    }

    /// Looks up and removes a key/value argument by key. If multiple arguments
    /// use the same key, the last one is returned (but all are removed).
    pub fn lookup(&mut self, key: &str) -> Option<&'a Argument> {
        let arg = self.args.iter().rev().find(|a| a.key() == Some(key)).copied();
        if arg.is_some() {
            self.args.retain(|a| a.key() != Some(key))
        }
        arg
    }

    /// Looks up and parses a key/value argument by key, removing it. If parsing
    /// errors, the argument is not removed.
    pub fn lookup_parse<T>(&mut self, key: &str) -> Result<Option<T>, Box<dyn Error>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let value =
            self.args.iter().rev().find(|a| a.key() == Some(key)).map(|a| a.parse()).transpose()?;
        if value.is_some() {
            self.args.retain(|a| a.key() != Some(key))
        }
        Ok(value)
    }

    /// Returns and removes the next key/value argument, if any.
    pub fn next_key(&mut self) -> Option<&'a Argument> {
        self.args.iter().position(|a| a.key().is_some()).map(|i| self.args.remove(i).unwrap())
    }

    /// Returns and removes the next positional argument, if any.
    pub fn next_pos(&mut self) -> Option<&'a Argument> {
        self.args.iter().position(|a| a.key().is_none()).map(|i| self.args.remove(i).unwrap())
    }

    /// Rejects any remaining arguments with an error.
    pub fn reject_rest(&self) -> Result<(), Box<dyn Error>> {
        if let Some(arg) = self.args.front() {
            return Err(format!("invalid argument '{arg}'").into());
        }
        Ok(())
    }

    /// Returns and removes all remaining arguments.
    pub fn rest(&mut self) -> Vec<&'a Argument> {
        self.args.drain(..).collect()
    }

    /// Returns and removes all remaining key/value arguments.
    pub fn rest_key(&mut self) -> Vec<&'a Argument> {
        let keyed: Vec<_> = self.args.iter().filter(|a| a.key().is_some()).copied().collect();
        if !keyed.is_empty() {
            self.args.retain(|a| a.key().is_none());
        }
        keyed
    }

    /// Returns and removes all remaining positional arguments.
    pub fn rest_pos(&mut self) -> Vec<&'a Argument> {
        let pos: Vec<_> = self.args.iter().filter(|a| a.key().is_none()).copied().collect();
        if !pos.is_empty() {
            self.args.retain(|a| a.key().is_some());
        }
        pos
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

    /// Basic tests of Argument.parse(). Not comprehensive, since it dispatches
    /// to core::str::parse().
    #[test]
    fn argument_parse() {
        assert_eq!(arg!("-1").parse::<i64>().unwrap(), -1_i64);
        assert_eq!(arg!("0").parse::<i64>().unwrap(), 0_i64);
        assert_eq!(arg!("1").parse::<i64>().unwrap(), 1_i64);

        assert_eq!(
            arg!("").parse::<i64>().unwrap_err().to_string(),
            "invalid argument '': cannot parse integer from empty string"
        );
        assert_eq!(
            arg!("foo").parse::<i64>().unwrap_err().to_string(),
            "invalid argument 'foo': invalid digit found in string"
        );

        assert!(!arg!("false").parse::<bool>().unwrap());
        assert!(arg!("true").parse::<bool>().unwrap());

        assert_eq!(
            arg!("").parse::<bool>().unwrap_err().to_string(),
            "invalid argument '': provided string was not `true` or `false`"
        );
    }

    /// Tests Command.consume_args(). ArgumentConsumer is tested separately.
    #[test]
    fn command_consume_args() {
        let cmd = cmd!("cmd foo key=value bar");
        assert_eq!(cmd.consume_args().rest(), vec![&cmd.args[0], &cmd.args[1], &cmd.args[2]]);
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

    /// Tests ArgumentConsumer.lookup().
    #[test]
    fn argument_consumer_lookup() {
        let cmd = cmd!("cmd value key=value foo=bar key=other");

        // lookup() returns None on unknown keys, including ones that match a
        // value argument.
        let mut args = cmd.consume_args();
        assert_eq!(args.lookup("unknown"), None);
        assert_eq!(args.lookup("value"), None);
        assert_eq!(args.rest().len(), 4);

        // lookup() removes duplicate keys, returning the last.
        let mut args = cmd.consume_args();
        assert_eq!(args.lookup("key"), Some(&cmd.args[3]));
        assert_eq!(args.rest(), vec![&cmd.args[0], &cmd.args[2]]);

        // lookup() removes single keys.
        let mut args = cmd.consume_args();
        assert_eq!(args.lookup("foo"), Some(&cmd.args[2]));
        assert_eq!(args.rest(), vec![&cmd.args[0], &cmd.args[1], &cmd.args[3]]);
    }

    /// Tests ArgumentConsumer.lookup_parse().
    #[test]
    fn argument_consumer_lookup_parse() {
        let cmd = cmd!("cmd value key=1 foo=bar key=2");

        // lookup_parse() returns None on unknown keys, including ones that
        // match a value argument.
        let mut args = cmd.consume_args();
        assert_eq!(args.lookup_parse::<String>("unknown").unwrap(), None);
        assert_eq!(args.lookup_parse::<String>("value").unwrap(), None);
        assert_eq!(args.rest().len(), 4);

        // lookup_parse() parses and removes duplicate keys, returning the last.
        let mut args = cmd.consume_args();
        assert_eq!(args.lookup_parse("key").unwrap(), Some(2));
        assert_eq!(args.rest(), vec![&cmd.args[0], &cmd.args[2]]);

        // lookup_parse() parses and removes single keys, with string parsing
        // being a noop.
        let mut args = cmd.consume_args();
        assert_eq!(args.lookup_parse("foo").unwrap(), Some("bar".to_string()));
        assert_eq!(args.rest(), vec![&cmd.args[0], &cmd.args[1], &cmd.args[3]]);

        // lookup_parse() does not remove arguments on parse errors, even with
        // duplicate keys.
        let mut args = cmd.consume_args();
        assert!(args.lookup_parse::<bool>("key").is_err());
        assert_eq!(args.rest(), vec![&cmd.args[0], &cmd.args[1], &cmd.args[2], &cmd.args[3]]);
    }

    /// Tests ArgumentConsumer.next(), next_pos(), and next_key().
    #[test]
    fn argument_consumer_next() {
        let cmd = cmd!("cmd foo key=1 key=2 bar");

        // next() returns references to all arguments and consumes them.
        let mut args = cmd.consume_args();
        assert_eq!(args.next(), Some(&cmd.args[0]));
        assert_eq!(args.next(), Some(&cmd.args[1]));
        assert_eq!(args.next(), Some(&cmd.args[2]));
        assert_eq!(args.next(), Some(&cmd.args[3]));
        assert_eq!(args.next(), None);
        assert!(args.rest().is_empty());

        // next_key() returns references to key/value arguments and consumes them.
        let mut args = cmd.consume_args();
        assert_eq!(args.next_key(), Some(&cmd.args[1]));
        assert_eq!(args.next_key(), Some(&cmd.args[2]));
        assert_eq!(args.next_key(), None);
        assert_eq!(args.next(), Some(&cmd.args[0]));
        assert_eq!(args.next(), Some(&cmd.args[3]));
        assert_eq!(args.next(), None);
        assert!(args.rest().is_empty());

        // next_pos() returns references to key/value arguments and consumes them.
        let mut args = cmd.consume_args();
        assert_eq!(args.next_pos(), Some(&cmd.args[0]));
        assert_eq!(args.next_pos(), Some(&cmd.args[3]));
        assert_eq!(args.next_pos(), None);
        assert_eq!(args.next(), Some(&cmd.args[1]));
        assert_eq!(args.next(), Some(&cmd.args[2]));
        assert_eq!(args.next(), None);
        assert!(args.rest().is_empty());
    }

    /// Tests ArgumentConsumer.reject_rest().
    #[test]
    fn argument_consumer_reject_rest() {
        // Empty args return Ok.
        let cmd = cmd!("cmd");
        assert!(cmd.consume_args().reject_rest().is_ok());

        // Positional argument fails. It does not consume the arg.
        let cmd = cmd!("cmd value");
        let mut args = cmd.consume_args();
        assert_eq!(args.reject_rest().unwrap_err().to_string(), "invalid argument 'value'");
        assert!(!args.rest().is_empty());

        // Key/value argument fails.
        let cmd = cmd!("cmd key=value");
        let mut args = cmd.consume_args();
        assert_eq!(args.reject_rest().unwrap_err().to_string(), "invalid argument 'key=value'");
        assert!(!args.rest().is_empty());
    }

    /// Tests ArgumentConsumer.rest(), rest_pos() and rest_key().
    #[test]
    fn argument_consumer_rest() {
        let cmd = cmd!("cmd foo key=1 key=2 bar");

        // rest() returns references to all arguments and consumes them.
        let mut args = cmd.consume_args();
        assert_eq!(args.rest(), vec![&cmd.args[0], &cmd.args[1], &cmd.args[2], &cmd.args[3]]);
        assert!(args.rest().is_empty());

        // rest_pos() returns and consumes positional arguments.
        let mut args = cmd.consume_args();
        assert_eq!(args.rest_pos(), vec![&cmd.args[0], &cmd.args[3]]);
        assert!(args.rest_pos().is_empty());
        assert_eq!(args.rest(), vec![&cmd.args[1], &cmd.args[2]]);

        // rest_key() returns and consumes key/value arguments.
        let mut args = cmd.consume_args();
        assert_eq!(args.rest_key(), vec![&cmd.args[1], &cmd.args[2]]);
        assert!(args.rest_key().is_empty());
        assert_eq!(args.rest(), vec![&cmd.args[0], &cmd.args[3]]);
    }
}
