mod command_attribute {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant {
            #[command]
            value: String,
        },
    }
}

mod name_value_attribute {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant {
            #[arg = "pos"]
            value: String,
        },
    }
}

mod pos_and_key {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant {
            #[arg(pos, key)]
            value: String,
        },
    }
}

mod unknown_argument_option {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant {
            #[arg(unknown)]
            value: String,
        },
    }
}

mod returns_first_field_error {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        Variant {
            #[arg(unknown)]
            first: String,
            #[command]
            second: String,
        },
    }
}

fn main() {}
