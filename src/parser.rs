use crate::command::{Argument, Block, Command};

use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, is_not, tag};
use nom::character::complete::{
    alphanumeric1, anychar, char, line_ending, not_line_ending, space0, space1,
};
use nom::combinator::{consumed, eof, opt, peek, recognize, value, verify};
use nom::error::{context, ParseError, VerboseError};
use nom::multi::{many0, many0_count, many_till};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::{Finish as _, IResult};

/// Parses the given goldenscript string into a list of command blocks.
pub(crate) fn parse(input: &str) -> Result<Vec<Block>, VerboseError<&str>> {
    blocks(input).finish().map(|(_, blocks)| blocks)
}

/// Parses a list of blocks until EOF.
fn blocks(input: &str) -> IResult<&str, Vec<Block>, VerboseError<&str>> {
    let (input, (blocks, _)) = many_till(context("block", block), eof)(input)?;
    Ok((input, blocks))
}

/// Parses a single block, consisting of a set of commands, a --- separator, and
/// the command output.
fn block(input: &str) -> IResult<&str, Block, VerboseError<&str>> {
    // Parse the command section, preserving the literal for output.
    let (input, (literal, commands)) = consumed(commands)(input)?;
    let block = Block { literal: literal.to_string(), commands };

    // If there were no commands, and we're at the end of the input, preserve
    // the literal as an empty block for output.
    if input.is_empty() && block.commands.is_empty() {
        return Ok((input, block));
    }

    // Parse the separator. There must be one.
    let (input, _) = context("separator", separator)(input)?;

    // Parse and skip the output section.
    let (input, _) = context("output", output)(input)?;

    Ok((input, block))
}

/// Parses the command section of a block. This consists of lines that are
/// either empty/blank, commands, or comments, up to the separator or EOF.
fn commands(mut input: &str) -> IResult<&str, Vec<Command>, VerboseError<&str>> {
    let mut commands = Vec::new();
    loop {
        // Skip empty/comment lines.
        if let (i, Some(_)) = opt(empty_or_comment_line)(input)? {
            input = i;
            continue;
        }

        // Detect premature EOF. This case must be handled by the caller.
        if input.is_empty() {
            return Ok((input, commands));
        }

        // If we hit a separator and we've seen at least 1 command, we're done.
        // Otherwise, we want to error while attempting to parse the command.
        if let (_, Some(_)) = peek(opt(separator))(input)? {
            if !commands.is_empty() {
                return Ok((input, commands));
            }
        }

        // Parse a command.
        let (i, command) = context("command", command)(input)?;
        commands.push(command);
        input = i;
    }
}

/// Parses a single command, consisting of a command name and optionally a set
/// of arguments (with or without values), prefix, and silencing parentheses.
/// Consumes the entire line, including any whitespace and comments at the end.
fn command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    // Look for a silencing (.
    let (input, maybe_silent) = opt(terminated(char('('), space0))(input)?;
    let silent = maybe_silent.is_some();

    // The command itself.
    let (input, prefix) =
        context("command prefix", opt(terminated(identifier, pair(tag(":"), space0))))(input)?;
    let (input, name) = context("command name", identifier)(input)?;
    let (mut input, args) = context("command arg", many0(preceded(space1, argument)))(input)?;

    // If silenced, look for the closing brace.
    if silent {
        (input, _) = preceded(space0, char(')'))(input)?;
    }

    // Ignore trailing whitespace and comments on this line.
    let (input, _) = space0(input)?;
    let (input, _) = opt(comment)(input)?;
    let (input, _) = line_ending(input)?;

    Ok((input, Command { name, args, prefix, silent }))
}

/// Parses a single command argument, consisting of an argument value and
/// optionally a key separated by =.
fn argument(input: &str) -> IResult<&str, Argument, VerboseError<&str>> {
    if let Ok((input, (key, value))) = separated_pair(identifier, tag("="), opt(string))(input) {
        return Ok((input, Argument { key: Some(key), value: value.unwrap_or_default() }));
    }
    let (input, value) = string(input)?;
    Ok((input, Argument { key: None, value }))
}

/// Parses a command/output separator: --- followed by a line ending.
fn separator(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    value((), terminated(tag("---"), alt((line_ending, eof))))(input)
}

/// Parses the command output following a --- separator, up to the first blank
/// line or EOF. This is typically two consecutive line endings, except the
/// special case where there is no output, i.e. the first character is a line
/// ending or EOF.
fn output(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    if let (input, Some(output)) = opt(alt((line_ending, eof)))(input)? {
        return Ok((input, output));
    }
    // TODO: many_till(anychar) is probably too expensive.
    recognize(many_till(anychar, pair(alt((line_ending, eof)), alt((line_ending, eof)))))(input)
}

/// Parses an identifier. This is any unquoted or quoted string that
/// isn't entirely empty.
fn identifier(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    verify(string, |s: &str| !s.is_empty())(input)
}

/// Parses a string, both quoted (' or ") and unquoted.
fn string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    alt((unquoted_string, quoted_string('\''), quoted_string('"')))(input)
}

/// An unquoted string can't contain whitespace, and can only contain
/// alphanumeric characters and some punctuation.
fn unquoted_string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    let (input, string) = recognize(pair(
        alt((alphanumeric1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_"), tag("-"), tag("."), tag("/")))),
    ))(input)?;
    Ok((input, string.to_string()))
}

/// A quoted string can contain anything, and respects common escape sequences.
/// It can be quoted using ' or ".
fn quoted_string<'a, E: ParseError<&'a str>>(
    quote: char,
) -> impl FnMut(&'a str) -> IResult<&'a str, String, E> {
    move |input| {
        let q = match quote {
            '\'' | '\"' => quote.to_string(),
            c => panic!("invalid quote character {c}"),
        };
        let q = q.as_str();

        // Because is_not in escaped_transform requires at least one matching
        // character, special-case the empty quoted string.
        if input.starts_with(&format!("{q}{q}")) {
            return Ok((&input[2..], String::new()));
        }

        let result = delimited(
            tag(q),
            escaped_transform(
                is_not(format!("\\{q}").as_str()),
                '\\',
                alt((
                    value("\'", tag("\'")),
                    value("\"", tag("\"")),
                    value("\\", tag("\\")),
                    value("\0", tag("0")),
                    value("\n", tag("n")),
                    value("\r", tag("r")),
                    value("\t", tag("t")),
                )),
            ),
            tag(q),
        )(input);
        result
    }
}

/// Parses a line that only contains whitespace and/or a comment.
fn empty_or_comment_line(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    verify(recognize(delimited(space0, opt(comment), alt((line_ending, eof)))), |line: &str| {
        !line.is_empty()
    })(input)
}

/// Parses a # or // comment until the end of the line/file (not inclusive).
fn comment(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(preceded(alt((tag("//"), tag("#"))), not_line_ending))(input)
}
