//! Derive macros for `goldenscript`.

mod expand;

use proc_macro::TokenStream;
use syn::{Error, parse_macro_input};

use expand::CommandExpander;

/// Generates a Goldenscript command parser for an enum.
///
/// Implements `TryFrom<&goldenscript::Command>` based on the enum variants
/// (commands) and their fields (arguments).
///
/// For usage details, see the [`goldenscript` documentation](https://docs.rs/goldenscript).
#[proc_macro_derive(Command, attributes(command, arg))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    CommandExpander::new().expand(input).unwrap_or_else(Error::into_compile_error).into()
}
