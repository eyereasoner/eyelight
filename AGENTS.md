# Working on eyelight

Eyelight is a small Rust relational reasoner with a native CLI and a WebAssembly playground. Read `SPEC.md` before changing language or proof behavior. Describe implemented behavior and keep new ambitions separate from current capabilities.

## Foundation

- The initial language is finite positive relational rules, with safe heads and structural value identity.
- Queries select output and never become inference evidence.
- The checker validates recorded applications without running inference search. Source assertions are assumed; checking does not prove completeness or absence.
- Preserve the C1–C9 contract in `SPEC.md`. C5, C6, and C8 are explicitly not applicable while their language features remain unsupported. Implement their checks when adding those features. Library callers must inspect `Report::valid()` or use `require_valid()`; `Ok(report)` can contain verification failures.
- Extensions such as arithmetic, negation, aggregation, existential heads, and quoted graphs require an explicit semantic design before implementation.
- Keep the core small and direct. Optimize only with measurements; do not claim superiority from implementation language or line count.

## Files and checks

All Rust files live directly in `src/`. `model.rs` holds values and clauses; `parser.rs` parses `.eye`; `engine.rs` evaluates; `proof.rs` generates and checks certificates. `main.rs` is the CLI and `wasm.rs` the browser bridge.

Run `./scripts/test-all` after code changes. Browser changes also need a real browser check at desktop and phone widths. Use `cargo clippy --target wasm32-unknown-unknown --lib -- -D warnings` for the Wasm library. Rebuild `pkg/` with `./scripts/build-playground` when Rust code changes.

Every meaningful behavior change needs a focused regression. Every proof-checking change needs a forged certificate the checker rejects. Preserve the exhaustive small-graph oracle test. Examples need a clear header describing the modeled situation, results, and language feature.

`Cargo.toml` is the version source. Follow the user's instructions for committing, pushing, and releasing. Review the complete working tree before any commit. Do not publish or create a remote repository without authorization.
