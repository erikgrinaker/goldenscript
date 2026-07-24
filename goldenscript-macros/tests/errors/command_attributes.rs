mod attribute_on_enum {
    #[derive(goldenscript_macros::Command)]
    #[command]
    enum Command {
        Variant,
    }
}

mod duplicate_name {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        First,
        #[command(name = "first")]
        Second,
    }
}

mod duplicate_name_precedes_field_error {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(name = "same")]
        First,
        #[command(name = "same")]
        Second(Option<String>, String),
    }
}

mod unknown_option {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(unknown)]
        Variant,
    }
}

mod name_value_attribute {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command = "variant"]
        Variant,
    }
}

mod attribute_on_variant {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[arg]
        Variant,
    }
}

mod returns_first_variant_error {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[arg]
        One,
        #[command(unknown)]
        Two,
    }
}

fn main() {}
