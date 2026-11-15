# acorn-rs

[![crates.io](https://img.shields.io/crates/v/acorn-rs.svg)](https://crates.io/crates/acorn-rs)
[![docs.rs](https://img.shields.io/docsrs/acorn-rs)](https://docs.rs/acorn-rs)
[![ci](https://github.com/harshmandan/acorn-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/harshmandan/acorn-rs/actions/workflows/ci.yml)

A Rust port of [acorn](https://github.com/acornjs/acorn) and
[@sveltejs/acorn-typescript](https://github.com/sveltejs/acorn-typescript),
carried over function for function.

It answers one question exactly the way acorn does: **does acorn accept
this text, and if not, which error does it throw, and where?** Same
message, same position, same choice of which error comes first.

> **This is a partial port.** It validates and finds where things end.
> It does not build a syntax tree yet. See the [checklist](#whats-ported).

## Why

Other Rust parsers (oxc, swc) are good parsers, but they are not acorn.
They word errors differently, report them at other positions, and
sometimes accept what acorn refuses or refuse what it accepts. Mapping
one parser's errors onto another's never ends: there is always one more
case.

If you need acorn's answer (for example, to report the same syntax errors
as the Svelte compiler), the only reliable way is to be acorn. This crate
started inside [svelte-check-native](https://github.com/harshmandan/svelte-check-native)
for that reason.

## Usage

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

Positions are byte offsets into the `&str` you pass. Inside, the parser
counts UTF-16 code units, as acorn does.

### API

| Function | acorn equivalent |
| :-- | :-- |
| `parse(source, Reading, Language)` | `acorn.parse(source)` |
| `parse_expression_at(source, pos, Language)` | `acorn.parseExpressionAt(source, pos)` — returns where it ends |
| `Text::new(source).parse_expression_at(..)` | same, for many reads of one text |
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

## Pinned versions

Each release follows exact commits of both upstreams. The test corpus
was produced by those commits, and the port agrees with it.

| Upstream | Version | Commit |
| :-- | :-- | :-- |
| acorn | 8.19.0 | [`0e0ece7`](https://github.com/acornjs/acorn/commit/0e0ece7cc7202624d4aca531be4b515b0114b5d0) |
| @sveltejs/acorn-typescript | 1.0.13, followed at its source (ahead of the release) | [`5e2e704`](https://github.com/sveltejs/acorn-typescript/commit/5e2e70417f655a70c592b55590ac24f9c995fb11) |

The same pins are in [`tests/corpus/versions.json`](tests/corpus/versions.json)
and in `Cargo.toml` under `[package.metadata.ported-from]`.

**Versioning.** While this is `0.x`:
- a **minor** bump (`0.1` → `0.2`) when the pins move or the API changes;
- a **patch** bump for fixes that keep the same pins.

Every release in [CHANGELOG.md](CHANGELOG.md) names its pins.

## What's ported

The settings are fixed to what the Svelte compiler passes acorn:
ECMAScript 2025, module code (always strict), parentheses kept in
expressions.

### acorn

- [x] Tokenizer: punctuators, numbers (separators, BigInt, legacy forms
      and their errors), strings and escapes, template literals, comments,
      Unicode identifiers (ID_Start / ID_Continue tables)
- [x] Regular expression validator, with its Unicode property tables
- [x] Statements, declarations, `for`/`for-in`/`for-of`/`for await`, labels
- [x] Expressions, operators and precedence, optional chaining, `new.target`,
      `import.meta`, dynamic `import()` with options, tagged templates
- [x] Patterns and destructuring, arrow parameters (converted the way acorn
      converts them)
- [x] Classes: fields, static blocks, private names (`#x`), accessors
- [x] Modules: `import` / `export`, string names, import attributes,
      undefined-export checks
- [x] Scopes and early errors: redeclarations, strict-mode names,
      `await` / `yield` placement, reserved words
- [x] Exact error messages and positions, and which error comes first
- [x] `#!` line at the start of a module
- [x] Deep nesting ends in acorn's "Not enough stack space" error instead
      of a crash
- [ ] **Syntax tree output** (ESTree nodes). Only what acorn itself looks
      at again is built.
- [ ] Configurable options. Today they are fixed as above, so these are
      missing: other `ecmaVersion`s, `sourceType: "script"` / sloppy mode,
      `allowReturnOutsideFunction`, `allowImportExportEverywhere`,
      `allowAwaitOutsideFunction`, `allowSuperOutsideMethod`,
      `allowReserved`, `checkPrivateFields`
- [ ] `using` / `await using` declarations (ecmaVersion 17+)
- [ ] `locations`, `ranges`, `sourceFile`, `directSourceFile`
- [ ] `onToken` / `onComment` / `onInsertedSemicolon` / `onTrailingComma`
- [ ] Public tokenizer (`acorn.tokenizer`)
- [ ] `loc` / `(line:column)` on errors (byte offset only)
- [ ] acorn's plugin system (TypeScript is a built-in switch)
- [ ] acorn-loose, acorn-walk

### @sveltejs/acorn-typescript

- [x] Types: annotations, type parameters and arguments, signatures,
      mapped, tuple, conditional and function types, type predicates
- [x] Declarations: interfaces, type aliases, enums, namespaces, `declare`,
      abstract classes, `import x = …`, `export =`, type-only imports and
      exports, `import defer`
- [x] Classes: modifiers and their order, parameter properties, index
      signatures, abstract members, `override`, `accessor`, decorators
- [x] Expressions: `as`, `satisfies`, `!`, `<T>expr`, type arguments on
      calls, and both readings it tries where an expression starts with `<`
- [x] Its own error table and positions, including the errors it raises
      at a column number taken for an offset
- [x] Its quirks, since they decide what is reported (for example, the
      import-attributes clause that steps over one token too many)
- [ ] **Syntax tree output** (TS nodes)
- [ ] JSX / TSX (the plugin's `jsx` option)
- [ ] `dts` mode (`.d.ts` reading)

## How it's tested

[`tests/corpus.rs`](tests/corpus.rs) holds the port to acorn itself:

- every input of acorn's own test suite (2,896) and of
  acorn-typescript's (3,188);
- each read four ways: as a component script, a module, an expression,
  and a single statement — 24,336 readings;
- each compared with what the pinned acorn answers: same message, same
  offset, or same end position.

They all agree. The one thing not compared is the exact offset of
"Not enough stack space", which acorn itself doesn't report the same
way twice.

The corpus is written by [`scripts/acorn-corpus-oracle.mjs`](scripts/acorn-corpus-oracle.mjs),
which runs the real acorn. It can also write **mutants** (each input with
tokens dropped, added, replaced or swapped) to find broken inputs the
test suites don't have:

```sh
MUTANTS_DIR=/tmp/m node scripts/acorn-corpus-oracle.mjs ../acorn scripts ../acorn-typescript
ACORN_CORPUS_EXTRA=/tmp/m/typescript-mutants.json cargo test --test corpus
```

## Moving to a newer acorn

```sh
git clone https://github.com/acornjs/acorn ../acorn
git clone https://github.com/sveltejs/acorn-typescript ../acorn-typescript
git -C ../acorn checkout <new commit>
git -C ../acorn-typescript checkout <new commit>
# set scripts/package.json's acorn to the new version, then:
(cd scripts && npm install)
# build acorn-typescript (see the header of the oracle script), then:
node scripts/acorn-corpus-oracle.mjs --repin ../acorn scripts ../acorn-typescript
cargo test
```

`--repin` rewrites the pins and the corpus together. Then port the
upstream changes until `cargo test` passes again. The script refuses
checkouts with local changes, and refuses to move acorn without
re-answering the TypeScript corpus too.

## Contributing

Contributions are welcome — especially the unchecked boxes above. A
syntax tree, more options, and JSX are the big ones.

A few rules keep the port honest:

- **Port, don't reinvent.** Follow acorn's function structure and call
  order. Which error wins depends on it.
- **Upstream is the spec.** If the port disagrees with acorn, the port is
  wrong — even where acorn looks like it has a bug.
- **Every behaviour change comes with a test.** Prefer a corpus entry
  from the oracle. For a case the corpus can't hold, add one to
  [`tests/reviewed_differences.rs`](tests/reviewed_differences.rs).
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and
  `cargo test` must pass.

Open an issue first for anything large, so we can agree on the shape.

## License

MIT. acorn and @sveltejs/acorn-typescript are MIT too; their notices are
in [LICENSE](LICENSE).
