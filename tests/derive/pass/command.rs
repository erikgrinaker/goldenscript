use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use goldenscript::Command;

#[derive(Command)]
enum TestCommand {
    Unit,
    #[command(name = "tuple")]
    Tuple(String, Option<u64>, Vec<u64>),
    Struct {
        #[arg(pos)]
        value: u64,
        #[arg(key)]
        required: u64,
        #[arg(key = "optional")]
        optional: Option<u64>,
        #[prefix]
        prefix: Option<char>,
        #[tag(name = "enabled")]
        flag: bool,
        #[tag]
        tags: BTreeSet<String>,
    },
    HashValues(HashSet<u64>),
    TreeValues(BTreeSet<u64>),
    HashProperties {
        #[arg(key)]
        values: HashMap<String, u64>,
    },
    TreeProperties {
        #[arg(key)]
        values: BTreeMap<String, u64>,
    },
}

fn convert(command: &Command) {
    let _: Result<TestCommand, Box<dyn std::error::Error>> = command.try_into();
}

fn main() {
    let _ = convert;
}
