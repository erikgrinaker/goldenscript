use std::collections::HashMap;

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
    /// Returns all key/value arguments as a HashMap. If duplicate keys are
    /// given, the last one is used.
    pub fn key_args(&self) -> HashMap<String, &Argument> {
        self.args.iter().filter_map(|a| a.key.as_ref().map(|k| (k.clone(), a))).collect()
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
    /// Parses the argument value as a T using core::str::parse(). Convenience
    /// method that returns a string error to ease error handling in a
    /// [`Runner`](crate::Runner).
    pub fn parse<T>(&self) -> Result<T, String>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        self.value.parse().map_err(|e| format!("invalid argument '{}': {e}", self.value))
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
        assert_eq!(
            cmd.key_args(),
            HashMap::from([("key".to_string(), &cmd.args[0]), ("foo".to_string(), &cmd.args[1])])
        );

        // Only positional arguments.
        cmd.args = vec![arg!("foo"), arg!("value")];
        assert_eq!(cmd.pos_args(), vec![&cmd.args[0], &cmd.args[1]]);
        assert!(cmd.key_args().is_empty());

        // Mixed arguments.
        cmd.args = vec![arg!("foo"), arg!("foo" => "bar"), arg!("value"), arg!("key" => "value")];
        assert_eq!(cmd.pos_args(), vec![&cmd.args[0], &cmd.args[2]]);
        assert_eq!(
            cmd.key_args(),
            HashMap::from([("foo".to_string(), &cmd.args[1]), ("key".to_string(), &cmd.args[3])])
        );

        // Duplicate key/value arguments.
        cmd.args = vec![arg!("key" => "1"), arg!("key" => "2"), arg!("key" => "3")];
        assert!(cmd.pos_args().is_empty());
        assert_eq!(cmd.key_args(), HashMap::from([("key".to_string(), &cmd.args[2])]));
    }

    /// Basic tests of Argument.parse(). Not comprehensive, since it dispatches
    /// to core::str::parse().
    #[test]
    fn argument_parse() {
        assert_eq!(arg!("-1").parse(), Ok(-1_i64));
        assert_eq!(arg!("0").parse(), Ok(0_i64));
        assert_eq!(arg!("1").parse(), Ok(1_i64));
        assert_eq!(
            arg!("").parse::<i64>(),
            Err("invalid argument '': cannot parse integer from empty string".to_string())
        );
        assert_eq!(
            arg!("foo").parse::<i64>(),
            Err("invalid argument 'foo': invalid digit found in string".to_string())
        );

        assert_eq!(arg!("false").parse(), Ok(false));
        assert_eq!(arg!("true").parse(), Ok(true));
        assert_eq!(
            arg!("").parse::<bool>(),
            Err("invalid argument '': provided string was not `true` or `false`".to_string())
        );
    }
}
