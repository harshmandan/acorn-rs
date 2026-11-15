//! The port against acorn itself.
//!
//! `corpus/javascript.json` holds every input of acorn's own test
//! suite (the programs it must accept and the ones it must reject),
//! each with what the pinned acorn (`corpus/versions.json`) does with
//! it when read the four ways the Svelte compiler reads text:
//!
//! - `script`: a component's `<script>`, never at offset zero, free to
//!   export a name it does not declare;
//! - `module`: a module file from its first character;
//! - `exprAt1`: `parseExpressionAt`, parentheses kept;
//! - `stmtAt1`: one statement, as the compiler reads a declaration tag.
//!
//! An entry is the error's message and byte offset, or where the
//! expression or statement ends, or nothing for a program read whole.
//! `scripts/acorn-corpus-oracle.mjs` writes the file from acorn.
//!
//! The port has to agree on every one. acorn decides which error comes
//! first and where it points by how it is written, so agreement on a
//! corpus this wide is what "the same parser" means here.

use acorn_rs::{Language, Position, Reading, STACK_EXHAUSTED, SyntaxError};

/// What acorn did. Where it failed with something other than a syntax
/// error at an offset, that failure's own message. Every entry answers
/// every reading: one that does not is a corpus written wrong.
fn want(value: &serde_json::Value, reading: &str, code: &str) -> String {
    let answer = &value[reading];
    assert!(answer.is_object(), "{code:?} has no {reading} answer");
    if let Some(message) = answer["crash"].as_str() {
        return format!("crash {message}");
    }
    match (answer["msg"].as_str(), answer["end"].as_u64()) {
        (Some(message), _) => format!("{message} @{}", answer["pos"]),
        (None, Some(end)) => format!("end {end}"),
        (None, None) => "ok".to_string(),
    }
}

/// What the port did with `text`, in the oracle's terms. `shift` is
/// how far into `text` the input starts: the oracle reads three of the
/// four ways one space into the text, and takes that space off again.
///
/// acorn-typescript raises a few errors at a column number that it
/// takes for an offset. The offset that names is worked out by the same
/// function the rest of the workspace reports those errors with.
fn got(text: &str, result: Result<Option<u32>, SyntaxError>, shift: u32) -> String {
    match result {
        Ok(Some(end)) => format!("end {}", end - shift),
        Ok(None) => "ok".to_string(),
        Err(error) => {
            let pos = match error.position {
                Position::Unknown => return format!("crash {}", error.message),
                Position::ColumnOf(pos) => acorn_rs::column_as_offset(text, pos),
                Position::Offset(pos) => pos,
            };
            format!("{} @{}", error.message, i64::from(pos) - i64::from(shift))
        }
    }
}

fn compare(corpus: &str, language: Language, at_least: usize) {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(corpus).unwrap_or_else(|e| panic!("the corpus is JSON: {e}"));
    assert!(cases.len() > at_least, "the corpus is all there");
    compare_cases(&cases, language);
}

fn compare_cases(cases: &[serde_json::Value], language: Language) {
    let mut failures = Vec::new();
    let mut total = 0;
    for case in cases {
        let code = case["code"]
            .as_str()
            .unwrap_or_else(|| panic!("a corpus entry without code: {case}"));
        // As the oracle reads: a component's script, a tag's expression
        // or statement is never at offset zero of what the parser is
        // given, so one space stands for the rest.
        let padded = format!(" {code}");
        let readings = [
            (
                "script",
                got(
                    &padded,
                    acorn_rs::parse(&padded, Reading::Script, language).map(|()| None),
                    1,
                ),
            ),
            (
                "module",
                got(
                    code,
                    acorn_rs::parse(code, Reading::Module, language).map(|()| None),
                    0,
                ),
            ),
            (
                "exprAt1",
                got(
                    &padded,
                    acorn_rs::parse_expression_at(&padded, 1, language).map(Some),
                    1,
                ),
            ),
            (
                "stmtAt1",
                got(
                    &padded,
                    acorn_rs::parse_statement_at(&padded, 1, language).map(Some),
                    1,
                ),
            ),
        ];
        for (reading, got) in readings {
            let want = want(case, reading, code);
            total += 1;
            // Where acorn runs out of stack is up to the JavaScript
            // engine and differs from one run to the next; only that it
            // does is compared.
            if got.starts_with(STACK_EXHAUSTED) && want.starts_with(STACK_EXHAUSTED) {
                continue;
            }
            if got != want {
                let shown: String = code.chars().take(200).collect();
                failures.push(format!(
                    "{reading} {shown:?}\n  got:  {got}\n  want: {want}"
                ));
            }
        }
    }
    if let Ok(path) = std::env::var("ACORN_CORPUS_DIFF") {
        let _ = std::fs::write(path, failures.join("\n"));
    }
    assert!(
        failures.is_empty(),
        "{} of {total} readings differ:\n{}",
        failures.len(),
        failures
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn port_agrees_with_acorn_on_its_own_tests() {
    compare(
        include_str!("corpus/javascript.json"),
        Language::JavaScript,
        2500,
    );
}

#[test]
fn port_agrees_with_acorn_typescript() {
    compare(
        include_str!("corpus/typescript.json"),
        Language::TypeScript,
        3000,
    );
}

/// A corpus from outside the repository: `ACORN_CORPUS_EXTRA` names a
/// file the oracle wrote, `ACORN_CORPUS_EXTRA_LANGUAGE` is `js` for
/// plain acorn's answers.
#[test]
fn port_agrees_on_a_corpus_given_by_the_environment() {
    let Ok(path) = std::env::var("ACORN_CORPUS_EXTRA") else {
        return;
    };
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path} is JSON: {e}"));
    assert!(!cases.is_empty(), "{path} holds a corpus");
    let language = match std::env::var("ACORN_CORPUS_EXTRA_LANGUAGE").as_deref() {
        Ok("js") => Language::JavaScript,
        _ => Language::TypeScript,
    };
    compare_cases(&cases, language);
}
