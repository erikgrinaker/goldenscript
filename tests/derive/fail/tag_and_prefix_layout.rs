use std::collections::{BTreeSet, HashSet};

use goldenscript::Command;

#[derive(Command)]
enum TestCommand {
    DuplicatePrefixes {
        #[prefix]
        first: String,
        #[prefix]
        second: String,
    },
    DuplicateTags {
        #[tag(name = "same")]
        first: bool,
        #[tag(name = "same")]
        second: bool,
    },
    DuplicateCollections {
        #[tag]
        first: BTreeSet<String>,
        #[tag]
        second: HashSet<String>,
    },
    NamedCollection {
        #[tag(name = "invalid")]
        values: Vec<String>,
    },
    TupleBool(#[tag] bool),
}

fn main() {}
