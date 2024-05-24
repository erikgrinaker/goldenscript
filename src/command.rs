use std::error::Error;

/// A block, consisting of multiple commands.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub(crate) struct Block {
    /// The commands in the block.
    pub commands: Vec<Command>,
    /// The literal string of the input commands. Used to generate the output.
    pub literal: String,
    /// The block's line number position in the script.
    pub line_number: u32,
}

/// A command.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct Command {
    /// The name of the command. Never empty.
    pub name: String,
    /// The command's arguments, in the given order.
    pub args: Vec<Argument>,
    /// The command prefix, if given.
    pub prefix: Option<String>,
    /// Silences the output of this command. This is handled automatically, the
    /// [`Runner`](crate::Runner) does not have to take this into account.
    pub silent: bool,
    /// The command's line number position in the script.
    pub line_number: u32,
}

impl Command {
    /// Returns all key/value arguments, in their original order.
    pub fn key_args(&self) -> Vec<&Argument> {
        self.args.iter().filter(|a| a.key.is_some()).collect()
    }

    /// Returns all positional arguments (no key), in their original order.
    pub fn pos_args(&self) -> Vec<&Argument> {
        self.args.iter().filter(|a| a.key.is_none()).collect()
    }
}

/// A command argument.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct Argument {
    /// The argument key, for `key=value` style arguments. Not guaranteed to be
    /// unique, the [`Runner`](crate::Runner) can handle this as desired.
    pub key: Option<String>,
    /// The argument value. Can be empty.
    pub value: String,
}

impl Argument {
    /// Returns a name for the argument -- either the key, if given, or value.
    pub fn name(&self) -> &str {
        match self.key.as_deref() {
            Some(key) => key,
            None => &self.value,
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
        self.value.parse().map_err(|e| format!("invalid argument '{}': {e}", self.value).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructs an Argument from a string value or string key => value.
    macro_rules! arg {
        ($value:expr) => {
            Argument { key: None, value: $value.to_string() }
        };
        ($key:expr => $value:expr) => {
            Argument { key: Some($key.to_string()), value: $value.to_string() }
        };
    }

    /// Tests Command.pos_args() and key_args().
    #[test]
    fn command_pos_key_args() {
        let mut cmd = Command {
            name: "name".to_string(),
            args: Vec::new(),
            prefix: None,
            silent: false,
            line_number: 0,
        };

        // Empty argument list.
        assert!(cmd.pos_args().is_empty());
        assert!(cmd.key_args().is_empty());

        // Only key/value arguments.
        cmd.args = vec![arg!("key" => "value"), arg!("foo" => "bar")];
        assert!(cmd.pos_args().is_empty());
        assert_eq!(cmd.key_args(), vec![&cmd.args[0], &cmd.args[1]]);

        // Only positional arguments.
        cmd.args = vec![arg!("foo"), arg!("value")];
        assert_eq!(cmd.pos_args(), vec![&cmd.args[0], &cmd.args[1]]);
        assert!(cmd.key_args().is_empty());

        // Mixed arguments.
        cmd.args = vec![arg!("foo"), arg!("foo" => "bar"), arg!("value"), arg!("key" => "value")];
        assert_eq!(cmd.pos_args(), vec![&cmd.args[0], &cmd.args[2]]);
        assert_eq!(cmd.key_args(), vec![&cmd.args[1], &cmd.args[3]]);

        // Duplicate key/value arguments.
        cmd.args = vec![arg!("key" => "1"), arg!("key" => "2"), arg!("key" => "3")];
        assert!(cmd.pos_args().is_empty());
        assert_eq!(cmd.key_args(), vec![&cmd.args[0], &cmd.args[1], &cmd.args[2]]);
    }

    /// Tests Argument.name().
    #[test]
    fn argument_name() {
        assert_eq!(arg!("value").name(), "value");
        assert_eq!(arg!("key" => "value").name(), "key");
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
}
