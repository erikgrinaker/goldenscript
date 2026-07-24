mod enum_attribute {
    use goldenscript::Command;

    #[derive(Command)]
    #[command]
    enum TestCommand {
        Variant,
    }
}

mod duplicate_commands {
    use goldenscript::Command;

    #[derive(Command)]
    enum TestCommand {
        First,
        #[command(name = "first")]
        Second,
    }
}

mod invalid_commands {
    use goldenscript::Command;

    #[derive(Command)]
    enum TestCommand {
        #[command(name = "")]
        Empty,
        #[command(unknown)]
        Unknown,
        #[command(name = "one", name = "two")]
        DuplicateOption,
        #[command]
        #[command]
        DuplicateAttribute,
        #[arg]
        Misplaced,
    }
}

fn main() {}
