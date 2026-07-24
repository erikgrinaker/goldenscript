#![warn(clippy::all)]

use std::error::Error;

/// Exposes hook calls as goldenscript output.
#[derive(Default)]
struct HookRunner {
    state: HookState,
}

/// Tracks hook state transitions, including start_script() and end_script(),
/// which can't return output.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum HookState {
    #[default]
    Start,
    Script,
    Block,
    Command,
    End,
}

impl HookRunner {
    /// Transitions to the next state, asserting that hooks are called in order.
    fn transition(&mut self, from: HookState, to: HookState) {
        assert_eq!(self.state, from);
        self.state = to;
    }
}

impl goldenscript::Runner for HookRunner {
    fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
        assert_eq!(self.state, HookState::Command);
        match command.name.as_str() {
            "echo" => {
                for arg in &command.args {
                    if arg.key().is_some() {
                        return Err("echo args can't have keys".into());
                    }
                }
                Ok(command.args.iter().map(|arg| arg.value()).collect::<Vec<_>>().join(" "))
            }
            "error" => Err("error".into()),
            "panic" => panic!("panic"),
            name => Err(format!("unknown command {name}").into()),
        }
    }

    fn start_script(&mut self) -> Result<(), Box<dyn Error>> {
        self.transition(HookState::Start, HookState::Script);
        Ok(())
    }

    fn end_script(&mut self) -> Result<(), Box<dyn Error>> {
        self.transition(HookState::Script, HookState::End);
        Ok(())
    }

    fn start_block(&mut self, block: &goldenscript::Block) -> Result<String, Box<dyn Error>> {
        self.transition(HookState::Script, HookState::Block);
        Ok(format!(
            "start_block: {}",
            block.commands.iter().map(|command| command.to_string()).collect::<Vec<_>>().join(", ")
        ))
    }

    fn end_block(&mut self, block: &goldenscript::Block) -> Result<String, Box<dyn Error>> {
        self.transition(HookState::Block, HookState::Script);
        Ok(format!(
            "end_block: {}",
            block.commands.iter().map(|command| command.to_string()).collect::<Vec<_>>().join(", ")
        ))
    }

    fn start_command(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
        self.transition(HookState::Block, HookState::Command);
        Ok(format!("start_command: {command}"))
    }

    fn end_command(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
        self.transition(HookState::Command, HookState::Block);
        Ok(format!("end_command: {command}"))
    }
}

#[test]
fn hooks() -> Result<(), Box<dyn Error>> {
    let mut runner = HookRunner::default();
    assert_eq!(runner.state, HookState::Start);
    goldenscript::run(&mut runner, "tests/hooks")?;
    assert_eq!(runner.state, HookState::End);
    Ok(())
}
