use std::collections::{BTreeMap, BTreeSet, HashMap};

use goldenscript::Command;

#[derive(Command)]
enum TestCommand {
    NestedOption(Option<Vec<String>>),
    NestedSequence(Vec<Option<String>>),
    NestedMap {
        #[arg(key)]
        values: HashMap<String, Vec<String>>,
    },
    NonStringMapKey {
        #[arg(key)]
        values: BTreeMap<u64, String>,
    },
    PrefixCollection {
        #[prefix]
        prefix: Vec<String>,
    },
    InvalidTag {
        #[tag]
        value: String,
    },
    InvalidTagElement {
        #[tag]
        values: BTreeSet<u64>,
    },
    NamedMap {
        #[arg(key = "name")]
        values: BTreeMap<String, String>,
    },
}

fn main() {}
