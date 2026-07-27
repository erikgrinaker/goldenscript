mod not_last {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(other)]
        Other(goldenscript::Command),
        Known,
    }
}

mod unit_variant {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(other)]
        Other,
    }
}

mod named_variant {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(other)]
        Other { command: goldenscript::Command },
    }
}

mod multiple_fields {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(other)]
        Other(goldenscript::Command, String),
    }
}

mod field_attribute {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(other)]
        Other(#[arg] goldenscript::Command),
    }
}

mod named_other {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(name = "other", other)]
        Other(goldenscript::Command),
    }
}

mod valued_other {
    #[derive(goldenscript_macros::Command)]
    enum Command {
        #[command(other = true)]
        Other(goldenscript::Command),
    }
}

fn main() {}
