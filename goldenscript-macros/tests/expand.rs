//! Tests macro expansion via goldenfiles under `expand`.

#[path = "../src/expand.rs"]
mod expand;

use std::io::Write;
use std::process::{Command, Stdio};

use proc_macro2::{Span, TokenStream};
use syn::{Error, Result, parse_quote};

use expand::CommandExpander;

/// Writes the given `TokenStream` to a goldenfile with the given name, formatted using rustfmt.
fn write_output(name: &str, output: TokenStream) -> Result<()> {
    let output = rustfmt(output)?;
    let mut mint = goldenfile::Mint::new("tests/expand");
    let mut golden = mint
        .new_goldenfile(format!("{name}.rs"))
        .map_err(|error| Error::new(Span::call_site(), error))?;
    golden.write_all(&output).map_err(|error| Error::new(Span::call_site(), error))?;
    Ok(())
}

fn rustfmt(output: TokenStream) -> Result<Vec<u8>> {
    let mut rustfmt = Command::new("rustfmt")
        .args(["--emit", "stdout", "--edition", "2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| Error::new(Span::call_site(), error))?;
    rustfmt
        .stdin
        .take()
        .expect("rustfmt stdin is piped")
        .write_all(output.to_string().as_bytes())
        .map_err(|error| Error::new(Span::call_site(), error))?;
    let output =
        rustfmt.wait_with_output().map_err(|error| Error::new(Span::call_site(), error))?;
    if !output.status.success() {
        return Err(Error::new(Span::call_site(), String::from_utf8_lossy(&output.stderr)));
    }
    Ok(output.stdout)
}

macro_rules! expand_test {
    ($name:ident, $input:item) => {
        #[test]
        fn $name() -> Result<()> {
            let input = parse_quote!($input);
            let output = CommandExpander::new().expand(input)?;
            write_output(stringify!($name), output)
        }
    };
}

expand_test! {
    command_names,
    enum Command {
        Error,
        HTTPRequest,
        GetURLValue,
        #[command(name = "renamed")]
        Other,
    }
}

expand_test! {
    data_types,
    enum Command {
        Scalar(bool),
        Optional(Option<usize>),
        Vec(Vec<String>),
        VecDeque(std::collections::VecDeque<String>),
        VecKeyValue(Vec<(String, u16)>),
        HashMap(std::collections::HashMap<String, f64>),
        HashSet(std::collections::HashSet<u16>),
        BTreeMap(std::collections::BTreeMap<String, char>),
        BTreeSet(std::collections::BTreeSet<i32>),
    }
}

expand_test! {
    variant_shapes,
    enum Command {
        Unit,
        Tuple(String, Vec<usize>),
        Struct {
            name: String,
            values: Vec<usize>,
        },
    }
}

expand_test! {
    argument_kinds,
    enum Command {
        Arguments {
            positional: String,
            #[arg(pos)]
            explicit_positional: u64,
            #[arg(key)]
            keyed: String,
            #[arg(key = "renamed")]
            named: Option<u64>,
            #[arg(many)]
            rest: Vec<String>,
            #[arg(key, many)]
            properties: std::collections::BTreeMap<String, u64>,
        },
        TupleKey(#[arg(key = "value")] u64),
        TupleMixed(
            #[arg(key = "first")]
            u64,
            String,
            #[arg(key = "second")]
            u64,
            bool,
        ),
        KeyManyThenPositional {
            #[arg(key, many)]
            properties: Properties<u64>,
            value: u64,
        },
    }
}

expand_test! {
    argument_options,
    enum Command {
        Optional {
            #[arg(optional)]
            optional: Option<u64>,
        },
        OptionalDefaults {
            #[arg(optional)]
            positional: u64,
            #[arg(key, optional)]
            keyed: u64,
        },
        Many {
            #[arg(many)]
            values: Values<String>,
            #[arg(key, many)]
            properties: Properties<u64>,
        },
        OptionalMany {
            #[arg(optional, many)]
            values: Values<String>,
            #[arg(key, optional, many)]
            properties: Properties<u64>,
        },
        NonGenericCollections {
            #[arg(optional, many)]
            values: Values,
            #[arg(key, optional, many)]
            properties: Properties,
        },
    }
}
