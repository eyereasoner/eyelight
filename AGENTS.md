# Working on eyelight

Eyelight is a small Rust relational reasoner with a native CLI and a WebAssembly
playground. Describe implemented behavior and distinguish it from design goals.
Keep diagnostics specific and user-facing text direct.

## Repository map

- All Rust files live directly in `src/`: `model.rs` holds values and clauses;
  `parser.rs` parses `.eye`; `engine.rs` evaluates; `proof.rs` generates and
  checks certificates. `lib.rs` exposes the API, `main.rs` the CLI, and
  `wasm.rs` the browser bridge.
- `examples/*.eye`: runnable programs with clear header comments describing
  the situation, result, and language feature. Do not mention project provenance.
- `tests/`: language, CLI, proof, worker, Pages, browser, and release checks.
- `playground.html`, `playground.css`, `playground.js`, `playground-worker.js`:
  browser UI and cancellable worker. `pkg/` is the committed WebAssembly build.
- `README.md`: guide. `SPEC.md`: language and proof contract.
- `scripts/`: checks, browser build, Pages staging, and version label syncing.
  `release`: release entry point. `.github/workflows/`: CI and Pages deployment.

## Language and proofs

Read `SPEC.md` before changing language or proof behavior. The language is finite
positive relational rules, with safe heads and structural value identity. Queries
select output and never become inference evidence. Arithmetic, negation,
aggregation, existential heads, and quoted graphs need an explicit semantic design
before implementation. Keep the core small and direct; optimize with measurements.

The checker validates recorded applications without running inference search.
Source assertions are assumed; checking does not prove completeness or absence.
Preserve the C1–C9 contract. C5, C6, and C8 are explicitly not applicable while
their language features remain unsupported; implement their checks when adding
those features. Library callers must inspect `Report::valid()` or use
`require_valid()`: `Ok(report)` can contain verification failures.

Every meaningful behavior change needs a focused regression. Every proof-checking
change needs a forged certificate the checker rejects. Preserve the exhaustive
small-graph oracle test. Keep implementation, specification, and examples consistent.

## Verification

```sh
cargo build --release --locked
./target/release/eyelight examples/socrates.eye
./target/release/eyelight --prove-and-check examples/socrates.eye
./scripts/test-all
cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings
./scripts/build-playground
git diff --check
```

Use focused tests while working, then run `./scripts/test-all` for code changes.
It checks formatting, Rust tests, native clippy, API docs, the browser worker,
Pages, and release tooling. Node.js 22 or later is required. Browser changes also
need a real browser check at desktop and phone widths; `tests/browser.mjs`
automates it through Chrome's debugging interface (see the guide).

Rebuild `pkg/` after Rust changes, and run Wasm clippy. The build requires
`wasm-pack` and the `wasm32-unknown-unknown` Rust target. Documentation-only changes
need a review and `git diff --check`; run Pages tests when changing website links
or staging. Avoid new dependencies or machinery without a concrete need.

## Website

`scripts/prepare-pages.mjs` stages the guide, specification, playground, and assets
in `_pages/`. The Pages workflow renders Jekyll into `_site/` and deploys it on
pushes to `main`. Both output directories are ignored.

Keep documentation links pointed at rendered Pages URLs; code and example links
can point at GitHub. Preserve the configured base path (normally `/eyelight/`),
`/playground/`, and `playground.html`. Run `node tests/pages.mjs` and verify the
rendered site in a browser after website changes. Version tests require the
committed Wasm, guide, specification, and playground to match `Cargo.toml`.

## Versions and releases

Follow the user's instructions for committing, pushing, and releasing. Commit to
`main`. Review the complete working tree before committing or releasing.
`Cargo.toml` is the version source; use the release script instead of bumping it
by hand:

```sh
./release                         # defaults to patch
./release patch "Release message"
./release minor
./release major
```

The optional message defaults to `Release v<version>`. The script switches to
`main`, pulls with `--ff-only`, bumps with `cargo set-version` (cargo-edit), syncs
version labels, rebuilds `pkg/`, runs checks, stages every change, commits, creates
an annotated tag, pushes branch and tag atomically, and creates a GitHub release
with `gh`. It needs authenticated Git and GitHub CLI access. No release workflow
is needed; a push to `main` runs CI and deploys Pages.

If a release fails, inspect which steps completed and finish that version; do not
blindly bump again. The guide documents publication recovery. Release tests use
temporary local repositories and a fake GitHub service, with no publication.
