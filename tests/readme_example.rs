//! The example in the README, kept true.

// The README shows the plain `unwrap_err()` a reader would write.
#![allow(clippy::unwrap_used)]

use acorn_rs::{Language, Reading, parse, parse_expression_at};

#[test]
fn readme_example() {
    let error = parse("let let = 1;", Reading::Module, Language::JavaScript).unwrap_err();
    assert_eq!(error.message, "The keyword 'let' is reserved");
    assert_eq!(error.pos, 4);

    assert!(parse("let x: number = 1;", Reading::Module, Language::TypeScript).is_ok());

    // Where the expression starting at offset 1 ends.
    assert_eq!(
        parse_expression_at("{a + b}", 1, Language::JavaScript),
        Ok(6)
    );
}
