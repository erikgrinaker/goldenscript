//! Tests errors emitted by `#[derive(Command)]`.

#![warn(clippy::all)]

#[test]
fn derive_errors() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/errors/*.rs");
}
