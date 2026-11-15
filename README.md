# acorn-rs

[![crates.io](https://img.shields.io/crates/v/acorn-rs.svg)](https://crates.io/crates/acorn-rs)
[![docs.rs](https://img.shields.io/docsrs/acorn-rs)](https://docs.rs/acorn-rs)
[![ci](https://github.com/harshmandan/acorn-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/harshmandan/acorn-rs/actions/workflows/ci.yml)

A Rust port of [acorn](https://github.com/acornjs/acorn) and
[@sveltejs/acorn-typescript](https://github.com/sveltejs/acorn-typescript),
carried over function for function.

Currently follows at:
- **acorn 8.19.0** ([`0e0ece7`](https://github.com/acornjs/acorn/commit/0e0ece7cc7202624d4aca531be4b515b0114b5d0))
- **@sveltejs/acorn-typescript 1.0.13** ([`5e2e704`](https://github.com/sveltejs/acorn-typescript/commit/5e2e70417f655a70c592b55590ac24f9c995fb11)),

> **This is a partial port.** It validates and finds where things end. It does not build a syntax tree yet. Contributions welcome — see [what's ported](#whats-ported).

```toml
[dependencies]
acorn-rs = "0.1"
```

```rust
use acorn_rs::{Language, Reading, parse, parse_expression_at};

let error = parse("let let = 1;", Reading::Module, Language::JavaScript).unwrap_err();
assert_eq!(error.message, "The keyword 'let' is reserved");
assert_eq!(error.pos, 4); // byte offset

assert!(parse("let x: number = 1;", Reading::Module, Language::TypeScript).is_ok());

// Where the expression starting at offset 1 ends.
assert_eq!(parse_expression_at("{a + b}", 1, Language::JavaScript), Ok(6));
```

## What's ported

Settings are fixed to what the Svelte compiler passes acorn: ECMAScript
2025, module code (always strict), parentheses kept in expressions.

☑ ported · ☐ not yet · — not part of that package

| Feature | acorn | acorn-typescript |
| :-- | :-: | :-: |
| Tokenizer: numbers, strings, escapes, templates, comments, Unicode identifiers | ☑ | ☑ |
| Regular expression validator, with Unicode property tables | ☑ | — |
| Statements, expressions, operators, optional chaining, `import()` | ☑ | ☑ |
| `as`, `satisfies`, `!`, `<T>expr`, type arguments on calls, both readings of a leading `<` | — | ☑ |
| Types: annotations, generics, mapped, tuple, conditional, function types, predicates | — | ☑ |
| Interfaces, type aliases, enums, namespaces, `declare` | — | ☑ |
| Patterns, destructuring, arrow parameters | ☑ | ☑ |
| Classes: fields, static blocks, private names | ☑ | ☑ |
| Class modifiers, parameter properties, index signatures, `abstract`, `override`, `accessor`, decorators | — | ☑ |
| Modules: `import` / `export`, string names, import attributes | ☑ | ☑ |
| Type-only imports and exports, `import x = …`, `export =`, `import defer` | — | ☑ |
| Scopes and early errors: redeclarations, strict-mode names, `await` / `yield` | ☑ | ☑ |
| Exact messages and positions, and which error comes first | ☑ | ☑ |
| Upstream quirks that decide what is reported (e.g. errors placed at a column taken for an offset) | ☑ | ☑ |
| `#!` line at the start of a module | ☑ | ☑ |
| "Not enough stack space" on deep nesting, instead of a crash | ☑ | ☑ |
| **Syntax tree output** (ESTree / TS nodes) | ☐ | ☐ |
| Configurable options: other `ecmaVersion`s, `sourceType: "script"`, `allow*` flags | ☐ | ☐ |
| `using` / `await using` declarations (ecmaVersion 17+) | ☐ | ☐ |
| `locations`, `ranges`, `onToken`, `onComment`, `loc` on errors | ☐ | ☐ |
| Public tokenizer (`acorn.tokenizer`) | ☐ | — |
| Plugin system (TypeScript is a built-in switch here) | ☐ | — |
| JSX / TSX | — | ☐ |
| `dts` mode (`.d.ts` files) | — | ☐ |
| acorn-loose, acorn-walk | ☐ | — |

## Tests

Every test passes. The main one runs every input from acorn's and
acorn-typescript's own test suites through the port, reads each one four
ways (component script, module, expression, single statement), and
compares the result with what the pinned upstream returns: same error
message and offset, or same end position.

| Suite | Inputs | Checks | Agree |
| :-- | --: | --: | --: |
| acorn's test suite | 2,896 | 11,584 | **11,584** |
| acorn-typescript's test suite | 3,188 | 12,752 | **12,752** |
| Deep nesting (stack limits) | — | 13 | **13** |
| Reviewed differences (past bugs) | — | 6 | **6** |
| Column offsets, README example | — | 2 | **2** |
| **Total** | | **24,357** | **24,357** |

8 of the corpus checks end in "Not enough stack space". For those, the
test checks that both sides run out of stack, not the exact offset,
because acorn itself reports a different offset from one run to the
next.

The corpus is written by [`scripts/acorn-corpus-oracle.mjs`](scripts/acorn-corpus-oracle.mjs),
which runs the real acorn. It can also generate mutants (inputs with
tokens dropped, added, replaced or swapped) for a wider run on demand.

## API

Positions are byte offsets into the `&str` you pass. Inside, the parser
counts UTF-16 code units, as acorn does.

| Function | acorn equivalent |
| :-- | :-- |
| `parse(source, Reading, Language)` | `acorn.parse(source)` |
| `parse_expression_at(source, pos, Language)` | `acorn.parseExpressionAt(source, pos)`, returning where it ends |
| `Text::new(source).parse_expression_at(..)` | the same, for many reads of one text |
| `parse_expression_within(source, pos, end, ..)` | `parseExpressionAt(source.slice(0, end), pos)` |
| `read_expression(source, ..)` | `parseExpressionAt(source, 0)`, plus where the next token starts |
| `parse_statement_at(source, pos, ..)` | one top-level statement at `pos` (Svelte's `parse_statement_at`) |
| `closing_brace(text)` | the `}` acorn's tokenizer would close a `{ … }` tag at |
| `regexp_pattern_error(pattern, flags)` | acorn's regex validator |
| `column_as_offset(source, pos)` | where acorn-typescript's column-as-offset errors point |

`Language` is `JavaScript` (acorn) or `TypeScript` (acorn with the
acorn-typescript plugin). `Reading::Module` reads a file from its start.
`Reading::Script` reads a Svelte component's `<script>`, which may export
names declared elsewhere.

## Contributing

Contributions are welcome, especially the ☐ rows above. A syntax tree,
configurable options and JSX are the big ones.

- **Port, don't reinvent.** Follow acorn's function structure and call
  order. Which error wins depends on it.
- **Upstream is the spec.** If the port disagrees with acorn, the port is
  wrong, even where acorn looks buggy.
- **Every behaviour change comes with a test.** Prefer a corpus entry
  from the oracle. Otherwise add one to
  [`tests/reviewed_differences.rs`](tests/reviewed_differences.rs).
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and
  `cargo test` must pass.
- The pinned upstream sources are submodules under `upstream/`
  (`git submodule update --init`). The tests don't need them; reading
  acorn's source and regenerating the corpus do.

Open an issue first for anything large.

## License

MIT. acorn and @sveltejs/acorn-typescript are MIT too; their notices are
in [LICENSE](LICENSE).
