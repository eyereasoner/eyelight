# Wheye guide

Version **0.1.14**. Run a Prolog subset in the native CLI or browser, get
answers, and inspect their independently checkable proofs. The
[specification](https://eyereasoner.github.io/wheye/SPEC/) defines the language
and the [C1–C9 contract](https://eyereasoner.github.io/wheye/SPEC/#8-verification).

## Run your first program

With Rust and Cargo installed:

```sh
cargo build --release --locked
./target/release/wheye examples/socrates.pl
./target/release/wheye --proof examples/socrates.pl
./target/release/wheye --prove-and-check examples/socrates.pl
cargo install --path . --locked
```

A minimal program selects Socrates as an answer:

```prolog
human(socrates).
mortal(Person) :- human(Person).
?- mortal(Who).
?- why.
:- assertion(mortal(socrates)).
```

`why` markers in existing programs are accepted without adding output. Use
`--proof` to see the reasoning. Assertions check an expected result and fail
the run if no solution exists; successful checks stay silent. They never assert facts. Opening a shared
playground program does not run it automatically; press **Run**.

## Explore the examples

All programs are directly in [examples/](https://github.com/eyereasoner/wheye/tree/main/examples/)
and selectable in the [playground](https://eyereasoner.github.io/wheye/playground/).
The collection includes lists, structural unification, recursive arithmetic,
Fibonacci at 10,000, graph algorithms, finite optimization, text processing,
scientific calculations, and classification chains. Source header comments
explain each example's assumptions and expected results. Saved transcripts in
`examples/output/` are reference outputs; the CLI's certificate and report
format is described below.

| Example | What it shows |
| --- | --- |
| [Socrates](https://github.com/eyereasoner/wheye/blob/main/examples/socrates.pl) | Queries, explanations, assertions |
| [Lists](https://github.com/eyereasoner/wheye/blob/main/examples/lists.pl) | Tabled list processing |
| [Aggregation](https://github.com/eyereasoner/wheye/blob/main/examples/aggregation.pl) | Counts, sums, minima, maxima |
| [List collection](https://github.com/eyereasoner/wheye/blob/main/examples/list-collection.pl) | Lists with repeated projected items |
| [Inventory](https://github.com/eyereasoner/wheye/blob/main/examples/inventory.pl) | Arithmetic followed by aggregation |
| [Airport paths](https://github.com/eyereasoner/wheye/blob/main/examples/path-discovery.pl) | Three routes across the full supplied airport graph |
| [Customer vouchers](https://github.com/eyereasoner/wheye/blob/main/examples/orders.pl) | All nine proof conditions, including witnesses |

The airport example has 7,698 airports and 37,274 directed flight links. It
finds three routes from Ostend to Prague with at most two intermediate stops.
Change `route_request/3` to ask for other endpoints or limits. The third answer
field is the requested limit; route text lists the actual airports. Repeated
airports are excluded, and an airport can have a zero-leg route to itself.
The example includes assertions for its supplied answers.

The packaged classification examples cover depths 10, 100, 1,000, 10,000, and
100,000. The first four include compatibility `why` markers and assertions; the largest selects
`test(true)` and is kept small enough to inspect or check without printing its
entire closure. Use files for large proofs:

```sh
./target/release/wheye -c examples/deep-taxonomy-100000.pl
./target/release/wheye -p examples/deep-taxonomy-100000.pl > /tmp/taxonomy.proof
```

Tests bound counted dispatch/matching work and proof size, including a
100,000-level chain. `node scripts/generate-taxonomy.mjs --check` verifies the
generated examples. `./scripts/bench-taxonomy` measures pipeline phases; run
it on a quiet machine without concurrent builds. Time and memory depend on
the program, machine, and allocator.

## Terms and arithmetic

Terms include distinct atoms, strings, exact integers, finite binary64 reals,
compound terms, and proper lists. `1` and `1.0` are different terms, while
`1 =:= 1.0` compares them numerically. Arithmetic expressions remain data until
`is/2`, a numeric comparison, or a numeric aggregate evaluates them.

```prolog
line(book, 12, 2).
total(Item, Amount) :- line(Item, Price, Quantity), Amount is Price * Quantity.
?- total(Item, Amount), Amount >= 20.
?- Result is 2^100.
?- point(X, [a|Rest]) = point(2, [a,b]).
```

Unification uses an occurs check. Each `_` is fresh; other names beginning
with `_` remain shared but are omitted from displayed bindings. Arithmetic
inputs must already be bound by preceding goals or a call. Invalid inputs,
division by zero, and non-finite results are errors. Integer arithmetic does
not wrap. See the [complete builtin tables](https://eyereasoner.github.io/wheye/SPEC/#41-built-ins-and-arithmetic).

## Collect completed answers

```prolog
purchase(tools, hammer, 5).
purchase(tools, saw, 5).
purchase(tools, saw, 5).
budget(Department, Count, Total) :-
    department(Department),
    aggregate_all(count, purchase(Department, Item, Price), Count),
    aggregate_all(sum(Price), purchase(Department, Item, Price), Total).
department(tools).
?- budget(Department, Count, Total).
```

The result is two items and a total of 10. Distinct producer tuples contribute
once; equal prices from different items contribute separately. Repeating the
same fact or proof path adds no contribution. `aggregate_all(list(Term), Goal,
Items)` retains equal projected elements from distinct producer tuples. Its
producer can be a parenthesized conjunction. Producer variables are local;
only the result binds outside, and earlier bindings constrain the producer.
Empty count and sum return zero; empty list collection returns `[]`; empty
minimum and maximum fail.

## Negation, tabling, and witnesses

```prolog
eligible(Person) :- registered(Person), \+ blocked(Person).
:- table fibonacci/2.
fibonacci(0, 0).
fibonacci(1, 1).
fibonacci(N, Value) :-
    N > 1, A is N-1, B is N-2,
    fibonacci(A, X), fibonacci(B, Y), Value is X+Y.
?- fibonacci(100, Value).
```

Negation requires a ground ordinary call and waits for its compatible
producers to complete. Negative and aggregate cycles are rejected. Clauses
that need caller inputs use demand-driven tables; a `table` declaration makes
all clauses of that predicate demand-driven. Variant calls share work and
recursive consumers resume as answers arrive. Tabling does not guarantee
termination for arbitrary programs. Limits and the playground's Stop button
bound a run.

`fresh(Witness)` is an additional explicit predicate with ordinary Prolog call
syntax. It can only end a forward rule and must bind a new output variable.
It creates an opaque typed witness for that source-rule firing. Source facts
cannot supply witness identities. The certificate checker verifies C8 for
the voucher example:

```sh
./target/release/wheye -c examples/orders.pl
```

## Keep and independently check evidence

```sh
./target/release/wheye -p examples/orders.pl > /tmp/orders.proof
./target/release/wheye --check-proof /tmp/orders.proof examples/orders.pl
```

There is one proof format, starting with `wheye-proof 1`. It lists supporting
source facts, rule applications, pure decisions, any required closed boundaries
and demanded-call patterns, and selected answer claims. `% Rule` comments show
the source rules used; comments are informative and the checker uses the supplied
source program. The proof is both readable and accepted by `--check-proof`.
Assertions execute during reasoning but are not exported as answer claims.
The checker validates the supplied evidence; it does not call the evaluator,
activate tables, or repair missing facts. C5 recomputes builtins and aggregates.
C6 requires complete finite models and transitively complete demanded calls;
omitting a derivable counterexample cannot establish absence or a smaller
aggregate. C8 checks witnesses, and C9 keeps queries and assertions out of
inference evidence.

Every report lists C1–C9. Conditions for unused features are not applicable.
The voucher example performs nonzero successful checks under all nine.
Source assertions remain assumptions. A valid certificate establishes support
for recorded claims; it does not authenticate facts, establish external-world
truth, or prove that every answer was selected. An empty certificate establishes
no positive claim.

## Use the command line

```sh
wheye [OPTIONS] FILE...
wheye [OPTIONS] -
wheye rules.pl data.pl
wheye --check-proof - rules.pl data.pl < /tmp/saved.proof
```

| Option | Output |
| --- | --- |
| `-p`, `--proof` | The readable, checkable proof |
| `-c`, `--prove-and-check` | Generate and independently check the proof |
| `--check-proof FILE` | Check saved evidence against the source |
| `--max-facts N` | Bound stored evidence |
| `--max-matches N` | Bound counted matching and finite-model work |
| `-v`, `--version` | Version |
| `-h`, `--help` | Usage |

Default output contains only answer bindings, `true.`, or `false.`, without query
headings, explanations, or successful assertion messages. Choose one output mode per invocation. Success returns status 0. Parse errors,
failed assertions, incomplete runs, and invalid proofs return status 1 with no
successful partial stdout. Keep source file order the same when checking;
source fact, rule, and query IDs are separate ordered lists.

## Embed the engine

```rust
use wheye::{check, parse, reason, Limits};
fn main() -> Result<(), wheye::Error> {
    let program = parse("human(socrates). mortal(X):-human(X). ?- mortal(Who).")?;
    let run = reason(&program, Limits::default())?;
    print!("{}", run.output(&program));
    let report = check(&program, &run.certificate(&program))?;
    report.require_valid()?;
    Ok(())
}
```

`Run` exposes stored evidence, query answers, counted work, and closed
boundaries. `Certificate::parse` reads saved evidence. `execute` offers the
same string interface as the browser. Both `check` and `check_with_limits`
return `Ok(report)` when semantic checking finishes, even for an invalid proof.
Inspect **`report.valid()`** or call **`report.require_valid()?`** before accepting
it. `report.conditions` lists the nine conditions; `report.failures` gives
specific diagnostics. `report.condition(wheye::Condition::C9)` selects one.
Malformed input and resource exhaustion return errors.

## Develop and verify

Rust, Cargo, Node.js 22 or later, `wasm-pack`, and the
`wasm32-unknown-unknown` Rust target are required for the complete checks.

```sh
./scripts/test-all
cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings
./scripts/build-playground
git diff --check
```

Tests run every packaged program and its assertions, check C1–C9, reject forged
certificates including omitted demanded calls, and preserve independent
exhaustive small-graph oracles. The browser tests use real WebAssembly and
check worker cancellation, sharing, desktop layout, and phone layout.

```sh
python3 -m http.server 8766 --bind 127.0.0.1
google-chrome --headless --remote-debugging-port=9226 --user-data-dir=/tmp/wheye-browser about:blank
node tests/browser.mjs
```

The browser engine runs in a worker. Programs remain local unless shared in a
URL. The committed `pkg/` must be rebuilt after Rust changes. Use the same
source version for CLI, documentation, playground, and Wasm.

## Make a release


From a reviewed checkout on `main`:

```sh
./release                         # patch: 0.1.0 → 0.1.1
./release minor                   # minor: 0.1.0 → 0.2.0
./release major                   # major: 0.1.0 → 1.0.0
./release patch "Improve proof diagnostics"
```

The first argument defaults to `patch`. The optional second argument becomes the commit message, annotated tag message, and GitHub release notes; it defaults to `Release v<version>`.

Install [cargo-edit](https://github.com/killercup/cargo-edit), [wasm-pack](https://github.com/drager/wasm-pack), and the [GitHub CLI](https://cli.github.com/), along with Rust, the `wasm32-unknown-unknown` target, and Node.js 22 or later. Sign in once with `gh auth login`. The script needs permission to push `main` and tags and create releases in the `origin` repository.

The script switches to `main`, pulls with `--ff-only`, bumps `Cargo.toml` with `cargo set-version`, and synchronizes the README, guide, specification, and playground version labels. It rebuilds WebAssembly, runs `./scripts/test-all` and Wasm clippy, then commits **all working-tree changes**, creates an annotated `v<version>` tag, pushes the branch and tag together, and creates the GitHub release. Review `git status` and `git diff` first. The push also starts CI and the Pages deployment.

A failed build or check stops before committing or publishing, leaving the local version changes available for inspection. Resolve the failure and finish that version rather than bumping again. If a commit and tag were created but publication stopped, retry only the unfinished steps, substituting the actual tag:

```sh
git push --atomic origin main v0.1.1
git tag --list v0.1.1 --format='%(contents)' |
  gh release create v0.1.1 --title=v0.1.1 --notes-file -
```

Skip the push if it already succeeded, and create the GitHub release only if it is still missing. The release contains GitHub's source archives. A separate tag workflow is not needed.

## Publish the website

The **Deploy GitHub Pages** workflow builds the site on pushes to `main` and can also be started manually. Set the repository's Pages source to **GitHub Actions**. The workflow checks the website and committed WebAssembly package, renders the guide and specification with Jekyll, and deploys the resulting static artifact.

Open the published [guide](https://eyereasoner.github.io/wheye/), [specification](https://eyereasoner.github.io/wheye/SPEC/), or [playground](https://eyereasoner.github.io/wheye/playground/). Documentation links in this repository point to those rendered pages. The `playground.html` route remains available too.

`GUIDE.md` supplies the guide at the site root; `SPEC.md` supplies the specification.
`node scripts/prepare-pages.mjs` prepares `_pages/` for the Jekyll build; `_site/` holds the rendered artifact. Both directories are ignored by Git. The Pages action supplies `PAGES_BASE_PATH`, so local links and assets work beneath `/wheye/` or a different configured prefix. The site uses the committed `pkg/`; changing the package version requires rebuilding it before publishing. The deployment check rejects a browser package whose exported version differs from `Cargo.toml`.

| File | Responsibility |
| --- | --- |
| `src/model.rs` | Values, facts, rules, queries, and validation |
| `src/parser.rs` | The `.pl` program syntax |
| `src/language.rs` | Safety, built-in modes, strata, and recursion restrictions |
| `src/semantics.rs` | Pure interpreted relations, aggregates, and finite-model matching |
| `src/engine.rs` | Indexed joins, agenda, limits, and answer rendering |
| `src/proof.rs` | Certificates and the separate checker |
| `src/main.rs` | Native command line and file I/O |
| `src/wasm.rs` | Browser bindings |
| `playground*.js` | UI and cancellable worker |
| `scripts/prepare-pages.mjs`, `scripts/pages-layout.html` | Pages staging and documentation layout |
| `.github/workflows/pages.yml` | Website build and deployment |
| `release`, `scripts/sync-version.mjs` | Version bump, browser rebuild, checks, and publication |

MIT licensed.
