mod named_map {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant {
            #[arg(key = "name", many)]
            values: std::collections::BTreeMap<String, String>,
        },
    }
}

mod optional_value {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant(#[arg(optional = true)] Option<String>),
    }
}

mod many_value {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant(#[arg(many(true))] Vec<String>),
    }
}

mod pos_value {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant(#[arg(pos = true)] String),
    }
}

mod key_list {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant(#[arg(key("value"))] String),
    }
}

fn main() {}
