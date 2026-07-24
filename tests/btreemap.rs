#![warn(clippy::all)]

use std::error::Error;
use std::fmt::Write as _;

/// A runner for BTreeMap tests. This is used as a documentation example.
#[derive(Default)]
struct BTreeMapRunner {
    map: std::collections::BTreeMap<String, String>,
}

impl goldenscript::Runner for BTreeMapRunner {
    fn run(&mut self, command: &goldenscript::Command) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        match command.name.as_str() {
            // get KEY: fetches the value of the given key, or None if it does not exist.
            "get" => {
                let mut args = command.consume_args();
                let key = args.next_pos().ok_or("key not given")?;
                args.reject_next()?;
                let value = self.map.get(key);
                writeln!(output, "get → {value:?}")?;
            }

            // insert KEY=VALUE...: inserts the given key/value pairs, returning the old value.
            "insert" => {
                let mut args = command.consume_args();
                while let Some((key, value)) = args.next_key() {
                    let old = self.map.insert(key.to_owned(), value.to_owned());
                    writeln!(output, "insert → {old:?}")?;
                }
                args.reject_next()?;
            }

            // range [FROM] [TO]: iterates over the key/value pairs in the range from..to.
            "range" => {
                use std::ops::Bound::*;
                let mut args = command.consume_args();
                let from =
                    args.next_pos().map(|value| Included(value.to_owned())).unwrap_or(Unbounded);
                let to =
                    args.next_pos().map(|value| Excluded(value.to_owned())).unwrap_or(Unbounded);
                args.reject_next()?;
                for (key, value) in self.map.range((from, to)) {
                    writeln!(output, "{key}={value}")?;
                }
            }

            name => return Err(format!("invalid command {name}").into()),
        };
        Ok(output)
    }
}

#[test]
fn btreemap() {
    goldenscript::run(&mut BTreeMapRunner::default(), "tests/btreemap")
        .expect("goldenscript failed")
}
