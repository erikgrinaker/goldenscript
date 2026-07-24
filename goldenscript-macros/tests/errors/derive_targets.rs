mod derive_struct {
    #[derive(goldenscript_macros::Command)]
    struct Command;
}

mod derive_union {
    #[derive(goldenscript_macros::Command)]
    union Command {
        value: u64,
    }
}

mod derive_generic {
    #[derive(goldenscript_macros::Command)]
    enum Command<T> {
        Variant(T),
    }
}

mod derive_empty {
    #[derive(goldenscript_macros::Command)]
    enum Command {}
}

fn main() {}
