mod required_after_optional {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant(Option<String>, String),
    }
}

mod after_many {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant(Vec<String>, String),
    }
}

mod after_map {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant {
            #[arg(key)]
            values: std::collections::BTreeMap<String, String>,
            #[arg(key)]
            required: String,
        },
    }
}

mod tuple_key {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant(#[arg(key)] String),
    }
}

mod duplicate_keys {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant {
            #[arg(key = "value")]
            first: String,
            #[arg(key = "value")]
            second: Option<String>,
        },
    }
}

fn main() {}
