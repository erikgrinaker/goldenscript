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

    /// Basic tests of Argument.parse(). Not comprehensive, since it dispatches
    /// to core::str::parse().
    #[test]
    fn argument_parse() {
        macro_rules! arg {
            ($s:expr) => {
                Argument { key: None, value: $s.to_string() }
            };
        }

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
