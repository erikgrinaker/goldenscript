use goldenscript::Command;

#[derive(Command)]
enum TestCommand {
    Conflicting {
        #[arg]
        #[tag]
        value: bool,
    },
    CommandOnField {
        #[command]
        value: String,
    },
    PrefixOptions {
        #[prefix(optional)]
        prefix: String,
    },
    ArgumentOptions {
        #[arg(pos, key)]
        value: String,
    },
    UnknownArgumentOption {
        #[arg(unknown)]
        value: String,
    },
    DuplicateArgumentOption {
        #[arg(key = "one", key = "two")]
        value: String,
    },
    EmptyKey {
        #[arg(key = "")]
        value: String,
    },
    EmptyTag {
        #[tag(name = "")]
        value: bool,
    },
}

fn main() {}
