# Working on acorn-rs

A function-for-function port of acorn and @sveltejs/acorn-typescript.
The goal is to give exactly the same answer as upstream: same error, same
offset, same choice of which error comes first. Upstream is the spec. If
the port disagrees with acorn, the port is wrong, even where acorn looks
buggy.

## Rules

- Port, don't reinvent. Keep acorn's function structure and call order.
- Every behaviour change comes with a test: a corpus entry from the
  oracle, or a case in `tests/reviewed_differences.rs`.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and
  `cargo test` must pass.
- Keep the README's "What's ported" table and test counts true.

## Pins

The upstream sources are git submodules, each at the commit the port
follows:

- `upstream/acorn` — https://github.com/acornjs/acorn
- `upstream/acorn-typescript` — https://github.com/sveltejs/acorn-typescript

Get them with `git submodule update --init`. Nothing in `cargo test`
needs them (the corpus is committed), so CI doesn't check them out, and
the crate's anchored `include` keeps them out of the published package.
Both submodules ignore untracked files, so the build the oracle needs
inside `upstream/acorn-typescript` doesn't show as a change.

The same commits are copied into `tests/corpus/versions.json`,
`[package.metadata.ported-from]` in `Cargo.toml`, and the README
overview. `scripts/package.json` pins the acorn npm version the oracle
answers with. `tests/pins.rs` fails if any of these disagree with the
submodules. It reads git's index, so a staged repin is checked before
it is committed.

Versioning while `0.x`: a minor bump when the pins move or the API
changes, a patch bump for fixes on the same pins. Every CHANGELOG entry
names its pins.

## Moving to newer upstream sources

```sh
git submodule update --init
git -C upstream/acorn fetch && git -C upstream/acorn checkout <new commit>
git -C upstream/acorn-typescript fetch && git -C upstream/acorn-typescript checkout <new commit>
```

1. Set `scripts/package.json`'s `acorn` to the new acorn version, then
   `(cd scripts && npm install)`.
2. Build acorn-typescript as the oracle script's header says (esbuild,
   with the installed acorn linked into its `node_modules`).
3. Rewrite the pins and the corpus together:

   ```sh
   node scripts/acorn-corpus-oracle.mjs --repin upstream/acorn scripts upstream/acorn-typescript
   ```

4. Port the upstream changes until `cargo test` passes again. The
   upstream diff between the old and new commits is the to-do list:
   `git -C upstream/acorn diff <old>..<new>` (the old commit is the one
   `git diff --submodule upstream` shows).
5. Update the pins in `Cargo.toml` and the README, add a CHANGELOG entry,
   bump the minor version. `git add upstream` records the new submodule
   commits; `cargo test --test pins` confirms every pin moved together.

The oracle refuses checkouts with local changes, and refuses to move
acorn without re-answering the TypeScript corpus too.

## Wider runs with mutants

The oracle can write mutants (inputs with tokens dropped, added, replaced
or swapped). They are too many to keep in the repository:

```sh
MUTANTS_DIR=/tmp/m node scripts/acorn-corpus-oracle.mjs upstream/acorn scripts upstream/acorn-typescript
ACORN_CORPUS_EXTRA=/tmp/m/typescript-mutants.json cargo test --test corpus
ACORN_CORPUS_EXTRA=/tmp/m/javascript-mutants.json ACORN_CORPUS_EXTRA_LANGUAGE=js cargo test --test corpus
```

`ACORN_CORPUS_DIFF=<file>` writes every disagreement to a file instead of
only the first 40 in the failure message.
