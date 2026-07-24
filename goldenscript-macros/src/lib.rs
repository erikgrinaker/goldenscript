//! Derive macros for `goldenscript`.
//!
//! See the
//! [command derive documentation](https://docs.rs/goldenscript/latest/goldenscript/#deriving-commands).

mod expand;

use proc_macro::TokenStream;
use syn::{Error, parse_macro_input};

use expand::CommandExpander;

/// Derives command parsing via `TryFrom<&goldenscript::Command>`.
#[proc_macro_derive(Command, attributes(command, arg))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    CommandExpander::new().expand(input).unwrap_or_else(Error::into_compile_error).into()
}
