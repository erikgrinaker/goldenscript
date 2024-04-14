/// A block, consisting of multiple commands.
#[derive(Debug, PartialEq)]
pub(crate) struct Block {
    /// The commands in the block.
    pub commands: Vec<Command>,
    /// The literal string of the input commands. Used to generate the output.
    pub literal: String,
}

/// A command.
#[derive(Debug, PartialEq)]
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
}

/// A command argument.
#[derive(Debug, PartialEq)]
pub struct Argument {
    /// The argument key, for `key=value` style arguments. Not guaranteed to be
    /// unique, the [`Runner`](crate::Runner) can handle this as desired.
    pub key: Option<String>,
    /// The argument value. Can be empty.
    pub value: String,
}
