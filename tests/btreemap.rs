#![warn(clippy::all)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Write as _;

/// Commands accepted by BTreeMapRunner.
#[derive(goldenscript::Command)]
enum BTreeMapCommand {
    /// Fetches the given keys.
    Get(Vec<String>),
    /// Inserts the given key/value pairs.
    Insert(Vec<(String, String)>),
    /// Scans the given [from, to) range.
    Range {
        #[arg(key)]
        from: Option<String>,
        #[arg(key)]
        to: Option<String>,
    },
}

/// A runner for BTreeMap tests. This is used as a documentation example.
#[derive(Default)]
struct BTreeMapRunner {
    map: BTreeMap<String, String>,
}

impl goldenscript::Runner for BTreeMapRunner {
    type Command = BTreeMapCommand;

    fn run(
        &mut self,
        command: &BTreeMapCommand,
        _: &goldenscript::Context,
    ) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        match command {
            BTreeMapCommand::Get(keys) => {
                for key in keys {
                    let value = self.map.get(key);
                    writeln!(output, "get {key:?} → {value:?}")?;
                }
            }

            BTreeMapCommand::Insert(entries) => {
                for (key, value) in entries {
                    let old = self.map.insert(key.clone(), value.clone());
                    writeln!(output, "insert {key:?} = {value:?} (was {old:?})")?;
                }
            }

            BTreeMapCommand::Range { from, to } => {
                use std::ops::Bound::*;
                let from = from.clone().map(Included).unwrap_or(Unbounded);
                let to = to.clone().map(Excluded).unwrap_or(Unbounded);
                writeln!(output, "range {from:?} → {to:?}")?;
                for (key, value) in self.map.range((from, to)) {
                    writeln!(output, "{key:?} = {value:?}")?;
                }
            }
        };
        Ok(output)
    }
}

#[test]
fn btreemap() {
    goldenscript::run(&mut BTreeMapRunner::default(), "tests/btreemap")
        .expect("goldenscript failed")
}
