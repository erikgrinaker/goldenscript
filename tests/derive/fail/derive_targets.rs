mod derive_struct {
    use goldenscript::Command;

    #[derive(Command)]
    struct TestCommand;
}

mod derive_union {
    use goldenscript::Command;

    #[derive(Command)]
    union TestCommand {
        value: u64,
    }
}

mod derive_generic {
    use goldenscript::Command;

    #[derive(Command)]
    enum TestCommand<T> {
        Value(T),
    }
}

mod derive_empty {
    use goldenscript::Command;

    #[derive(Command)]
    enum TestCommand {}
}

fn main() {}
