use crate::parser::parse;
use crate::Command;

use std::io::{Error, ErrorKind, Write as _};

/// Runs goldenscript commands, returning their output.
pub trait Runner {
    /// Runs a goldenscript command, returning its output, or a string error if
    /// the command failed. To test error cases, return an `Ok` result
    /// containing e.g. the error message as output.
    fn run(&mut self, command: &Command) -> Result<String, String>;

    /// Called at the start of a goldenscript. Used e.g. for initial setup.
    /// Can't return output, since it's not called in the context of a block.
    fn start_script(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Called at the end of a goldenscript. Used e.g. for state assertions.
    /// Can't return output, since it's not called in the context of a block.
    fn end_script(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Called at the start of a block. Used e.g. to output initial state.
    /// Any output is prepended to the block's output.
    fn start_block(&mut self) -> Result<String, String> {
        Ok(String::new())
    }

    /// Called at the end of a block. Used e.g. to output final state.
    /// Any output is appended to the block's output.
    fn end_block(&mut self) -> Result<String, String> {
        Ok(String::new())
    }
}

/// Runs a goldenscript at the given path.
///
/// Panics if the script output differs from the current input file. Errors on
/// IO, parser, or runner failure. If the environment variable
/// `UPDATE_GOLDENFILES=1` is set, the new output file will replace the input
/// file.
pub fn run<R: Runner, P: AsRef<std::path::Path>>(runner: &mut R, path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    let Some(dir) = path.parent() else {
        return Err(Error::new(ErrorKind::InvalidInput, format!("invalid path '{path:?}'")));
    };
    let Some(filename) = path.file_name() else {
        return Err(Error::new(ErrorKind::InvalidInput, format!("invalid path '{path:?}'")));
    };

    let input = std::fs::read_to_string(dir.join(filename))?;
    let output = generate(runner, &input)?;

    goldenfile::Mint::new(dir).new_goldenfile(filename)?.write_all(output.as_bytes())
}

/// Generates output for a goldenscript input, without comparing them.
pub(crate) fn generate<R: Runner>(runner: &mut R, input: &str) -> std::io::Result<String> {
    let mut output = String::with_capacity(input.len()); // common case: output == input

    // Detect end-of-line format.
    let eol = match input.find("\r\n") {
        Some(_) => "\r\n",
        None => "\n",
    };

    // Parse the script.
    let blocks = parse(input).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!(
                "parse error at line {} column {} for {:?}:\n{}\n{}^",
                e.input.location_line(),
                e.input.get_column(),
                e.code,
                String::from_utf8_lossy(e.input.get_line_beginning()),
                ' '.to_string().repeat(e.input.get_utf8_column() - 1)
            ),
        )
    })?;

    // Call the start_script() hook.
    runner.start_script().map_err(|e| Error::new(ErrorKind::Other, e))?;

    for (i, block) in blocks.iter().enumerate() {
        // There may be a trailing block with no commands if the script has bare
        // comments at the end. If so, just retain its literal contents.
        if block.commands.is_empty() {
            output.push_str(&block.literal);
            continue;
        }

        // Process each block of commands and accumulate their output.
        let mut block_output = String::new();

        // Call the start_block() hook.
        block_output.push_str(&ensure_eol(
            runner.start_block().map_err(|e| Error::new(ErrorKind::Other, e))?,
            eol,
        ));

        for command in &block.commands {
            // Execute the command.
            let mut command_output =
                runner.run(command).map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;

            // Silence the output if requested.
            if command.silent {
                command_output = "".to_string();
            }

            // Prefix output lines if requested.
            if let Some(prefix) = &command.prefix {
                command_output = format!(
                    "{prefix}: {}{eol}",
                    command_output.replace('\n', &format!("\n{prefix}: "))
                );
            }

            // Make sure the command output has a trailing newline, unless empty.
            command_output = ensure_eol(command_output, eol);

            block_output.push_str(&command_output);
        }

        // Call the end_block() hook.
        block_output.push_str(&ensure_eol(
            runner.end_block().map_err(|e| Error::new(ErrorKind::Other, e))?,
            eol,
        ));

        // If the block output contains blank lines, use a > prefix for it.
        //
        // We'd be better off using regular expressions here, but don't want to
        // add a dependency just for this.
        if block_output.starts_with('\n')
            || block_output.starts_with("\r\n")
            || block_output.contains("\n\n")
            || block_output.contains("\n\r\n")
        {
            block_output = format!("> {}", block_output.replace('\n', "\n> "));
            // We guarantee above that block output ends with a newline, so we
            // remove the "> " at the end of the output.
            block_output.pop();
            block_output.pop();
        }

        // Add the resulting block to the output. If this is not the last block,
        // also add a newline separator.
        output.push_str(&format!("{}---{eol}{}", block.literal, block_output));
        if i < blocks.len() - 1 {
            output.push_str(eol);
        }
    }

    // Call the end_script() hook.
    runner.end_script().map_err(|e| Error::new(ErrorKind::Other, e))?;

    Ok(output)
}

/// Appends a newline if the string is not empty and doesn't already have one.
fn ensure_eol(mut s: String, eol: &str) -> String {
    if let Some(c) = s.chars().next_back() {
        if c != '\n' {
            s.push_str(eol)
        }
    }
    s
}

// NB: most tests are done as goldenscripts under tests/.
#[cfg(test)]
mod tests {
    use super::*;

    /// A runner which simply counts the number of times its hooks are called.
    #[derive(Default)]
    struct HookRunner {
        start_script_count: usize,
        end_script_count: usize,
        start_block_count: usize,
        end_block_count: usize,
    }

    impl Runner for HookRunner {
        fn run(&mut self, _: &Command) -> Result<String, String> {
            Ok(String::new())
        }

        fn start_script(&mut self) -> Result<(), String> {
            self.start_script_count += 1;
            Ok(())
        }

        fn end_script(&mut self) -> Result<(), String> {
            self.end_script_count += 1;
            Ok(())
        }

        fn start_block(&mut self) -> Result<String, String> {
            self.start_block_count += 1;
            Ok(String::new())
        }

        fn end_block(&mut self) -> Result<String, String> {
            self.end_block_count += 1;
            Ok(String::new())
        }
    }

    /// Tests that runner hooks are called as expected.
    #[test]
    fn hooks() {
        let mut runner = HookRunner::default();
        generate(
            &mut runner,
            r#"
command
---

command
---
"#,
        )
        .unwrap();

        assert_eq!(runner.start_script_count, 1);
        assert_eq!(runner.end_script_count, 1);
        assert_eq!(runner.start_block_count, 2);
        assert_eq!(runner.end_block_count, 2);
    }
}
