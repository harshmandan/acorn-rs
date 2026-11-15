//! The library's own surface: its error type and how it takes offsets.

#![allow(clippy::unwrap_used)]

use acorn_rs::{
    Language, NoClosingBrace, Position, Reading, SyntaxError, Text, closing_brace, parse,
    parse_expression_at, parse_expression_within,
};

#[test]
fn a_syntax_error_is_an_error() {
    fn read() -> Result<(), Box<dyn std::error::Error>> {
        parse("let let = 1;", Reading::Module, Language::JavaScript)?;
        Ok(())
    }
    let error = read().unwrap_err();
    assert_eq!(
        error.to_string(),
        "The keyword 'let' is reserved (at byte 4)"
    );
}

#[test]
fn an_offset_past_the_end_reads_from_the_end() {
    let error = |result: Result<u32, SyntaxError>| result.unwrap_err().position;
    let at_end = Position::Offset(3);
    assert_eq!(
        error(parse_expression_at("abc", 10, Language::JavaScript)),
        at_end
    );
    assert_eq!(
        error(Text::new("abc").parse_expression_at(10, Language::JavaScript)),
        at_end
    );
    assert_eq!(
        error(parse_expression_within("abc", 10, 15, Language::JavaScript)),
        at_end
    );
    assert_eq!(
        parse_expression_within("a+b; c", 0, 99, Language::JavaScript),
        Ok(3)
    );
}

#[test]
fn closing_brace_says_why_it_found_none() {
    assert_eq!(closing_brace("a + b } c"), Ok(6));
    assert_eq!(closing_brace("a + {b}"), Err(NoClosingBrace::EndOfText));
    assert_eq!(closing_brace("a /* b"), Err(NoClosingBrace::EndOfText));
    assert_eq!(closing_brace("a # b }"), Err(NoClosingBrace::Unreadable));
}
