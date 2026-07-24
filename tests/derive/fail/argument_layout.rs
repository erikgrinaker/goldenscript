use std::collections::{BTreeMap, BTreeSet};

use goldenscript::Command;

#[derive(Command)]
enum TestCommand {
    RequiredAfterOptional(Option<String>, String),
    RequiredKeyAfterOptional {
        #[arg(key)]
        optional: Option<String>,
        #[arg(key)]
        required: String,
    },
    AfterVariadic(Vec<String>, String),
    KeyAfterMap {
        #[arg(key)]
        values: BTreeMap<String, String>,
        #[arg(key)]
        required: String,
    },
    SequenceKey {
        #[arg(key)]
        values: BTreeSet<String>,
    },
    PositionalMap(#[arg(pos)] BTreeMap<String, String>),
    TupleKey(#[arg(key)] String),
    DuplicateKeys {
        #[arg(key = "value")]
        first: String,
        #[arg(key = "value")]
        second: Option<String>,
    },
}

fn main() {}
