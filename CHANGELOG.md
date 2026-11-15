# Changelog

## 0.2.0

Pins:
- acorn 8.19.0 at `0e0ece7cc7202624d4aca531be4b515b0114b5d0`
- @sveltejs/acorn-typescript 1.0.13, at its source `5e2e70417f655a70c592b55590ac24f9c995fb11`

The pinned sources are now submodules under `upstream/`.

Breaking:
- `SyntaxError` has a `position` (`Position::Offset`, `ColumnOf` or
  `Unknown`) in place of `pos`, `column_of` and `no_position`, and is
  `#[non_exhaustive]`. It implements `Display` and `std::error::Error`.
- `closing_brace` says why it found no `}` with `NoClosingBrace`
  instead of a `bool`.
- `REGEXP_STACK_EXHAUSTED` is now `STACK_EXHAUSTED`, the message for
  any input nested too deep.
- `parse_expression_within` takes offsets past the end of the text as
  its end, as `parse_expression_at` does.

Fixed:
- A TypeScript member chain tens of thousands of links long followed by
  `<`, or inside parentheses before `?`, overflowed the stack and
  aborted the process. acorn-typescript reads it.
- A long run of U+0085 in TypeScript was read without limit, so it was
  accepted where acorn-typescript runs out of stack, and could overflow
  the stack.
- `parse_expression_at`, `read_expression` and `parse_statement_at`
  now skip a `#!` line when they read from offset zero, as acorn does.
- Declarations, exports, private names and parameters are looked up by
  hash. Tens of thousands of them in one scope took seconds; they are
  now linear.
- Fewer allocations per token.

## 0.1.0

First release, split out of
[svelte-check-native](https://github.com/harshmandan/svelte-check-native)
with its history.

Pins:
- acorn 8.19.0 at `0e0ece7cc7202624d4aca531be4b515b0114b5d0`
- @sveltejs/acorn-typescript 1.0.13, at its source `5e2e70417f655a70c592b55590ac24f9c995fb11`

Agrees with both on all 24,336 corpus readings (see the README for what
that covers and what is not ported yet).
