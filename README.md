<img src="assets/eyelight.svg" alt="" width="80" height="80" align="left">

# eyelight

A small relational reasoner with explanations and independently checked proof certificates.

<br clear="left">

**Version 0.1.2** · [Guide](https://eyereasoner.github.io/eyelight/) · [Language](https://eyereasoner.github.io/eyelight/SPEC/) · [Playground](https://eyereasoner.github.io/eyelight/playground/) · [Examples](https://github.com/eyereasoner/eyelight/tree/main/examples/)

Give eyelight facts and rules, then ask what follows:

```prolog
human(socrates).
mortal(Person) :- human(Person).

?- mortal(Who).
```

```text
?- mortal(Who).
Who = socrates.
```

Eyelight's `.eye` language uses familiar clause notation. It implements **Datalog-style rules with stratified negation, aggregation, explicit built-ins, and scoped fresh witnesses**. It does not parse N3 or implement general Prolog. The [specification](https://eyereasoner.github.io/eyelight/SPEC/) describes the complete language.

## Run your first program

With Rust and Cargo installed:

```sh
cargo build --release --locked
./target/release/eyelight examples/socrates.eye
./target/release/eyelight --explain examples/socrates.eye
./target/release/eyelight --prove-and-check examples/socrates.eye
```

Install from this checkout if you want `eyelight` on your path:

```sh
cargo install --path . --locked
```

The native engine has no third-party runtime dependencies. The browser build uses `wasm-bindgen` to expose the same Rust implementation.

## Try a connection

This program finds indirect connections as well as direct ones:

```prolog
link(library, park).
link(park, station).
link(station, park).

path(From, To) :- link(From, To).
path(From, To) :- path(From, Via), link(Via, To).

?- path(library, Where).
```

The answers are `park` and `station`. The cycle does not create duplicate answers or infinite recursion: the evaluator stops when it has no new facts to add.

| Example | What it shows |
| --- | --- |
| [Socrates](https://github.com/eyereasoner/eyelight/blob/main/examples/socrates.eye) | A first conclusion and query |
| [Paths](https://github.com/eyereasoner/eyelight/blob/main/examples/paths.eye) | Recursive reachability with cycles |
| [Family](https://github.com/eyereasoner/eyelight/blob/main/examples/family.eye) | Joins and shared variables |
| [Values](https://github.com/eyereasoner/eyelight/blob/main/examples/values.eye) | Distinct atoms, strings, and integers |
| [RDF-shaped data](https://github.com/eyereasoner/eyelight/blob/main/examples/rdf.eye) | IRIs represented in an ordinary triple relation |
| [Price calculations](https://github.com/eyereasoner/eyelight/blob/main/examples/arithmetic.eye) | Arithmetic, comparisons, and string built-ins |
| [Outside the network](https://github.com/eyereasoner/eyelight/blob/main/examples/negation.eye) | Stratified negation after recursive reachability |
| [Department budgets](https://github.com/eyereasoner/eyelight/blob/main/examples/aggregation.eye) | Grouped counts, sums, minima, and maxima |
| [Fresh tickets](https://github.com/eyereasoner/eyelight/blob/main/examples/witnesses.eye) | Witnesses scoped to individual rule firings |
| [Customer vouchers](https://github.com/eyereasoner/eyelight/blob/main/examples/orders.eye) | Every C1–C9 condition actively checked |

## Keep the evidence

```sh
./target/release/eyelight -p examples/socrates.eye > /tmp/socrates.proof
./target/release/eyelight --check-proof /tmp/socrates.proof examples/socrates.eye
```

The certificate is small enough to read:

```text
eyelight-proof 1
step 1 source 1 : human(socrates).
step 2 rule 1 [1] : mortal(socrates).
answer 1 [2]
```

Step 1 cites the first source fact. Step 2 applies the first source rule using step 1. The answer selects step 2 through the first query. The checker takes facts, rules, and queries from the supplied program, then validates each application with consistent bindings. It does not run the forward evaluator.

Checking reports:

```text
checked: 2 steps, 1 claim
1 source fact; 1 rule application
Conditions:
  C1 resolution: 2 checked, 0 failed
  C2 well-foundedness: 1 checked, 0 failed
  C3 justification: 3 checked, 0 failed
  C4 coverage: 5 checked, 0 failed
  C5 re-decision: not applicable (program uses no built-ins or aggregates)
  C6 boundary consistency: not applicable (program uses no negation or aggregation)
  C7 relevance: 2 checked, 0 failed
  C8 witness integrity: not applicable (program uses no fresh witnesses)
  C9 query isolation: 1 checked, 0 failed
Source assertions are assumed. Answer completeness is not checked.
```

A valid certificate establishes **support for its recorded claims relative to the source program**. Input facts are assumptions. The checker does not authenticate their authors or establish that every answer was found. A failed query prints `false.` but produces no claim proving that failure. An empty certificate establishes no answers.

The [C1–C9 contract](https://eyereasoner.github.io/eyelight/SPEC/#8-verification) defines the requirements and counting units. All nine conditions have implementations. Applicability depends on the program: Socrates needs no built-in decisions, closed-world boundaries, or witnesses, so C5, C6, and C8 report not applicable there. The voucher example exercises all nine with nonzero check counts.

## Calculate, exclude, and collect

Built-ins are explicit interpreted relations, like potentially infinite fact boxes. Bound inputs select a tuple or determine an output:

```prolog
?- @add(2, 3, Total), @ge(Total, 5).
```

Supported operations include checked integer arithmetic, comparisons, structural equality, string concatenation, and Unicode string length. Inputs must already be bound; undefined arithmetic tuples fail rather than wrapping. Ordinary names such as `add` remain ordinary relations.

Negation and aggregation inspect completed lower strata:

```prolog
customer(ada).
customer(bea).
line(ada, book, 12).
line(ada, pen, 3).
line(bea, map, 7).
blocked(bea).

subtotal(Customer, Total) :-
    customer(Customer),
    sum { Price, Item : line(Customer, Item, Price) } = Total.

eligible(Customer, Total) :-
    subtotal(Customer, Total), not blocked(Customer), @ge(Total, 10).

voucher(Customer, Voucher, Total) :-
    eligible(Customer, Total), @fresh(Voucher).

?- voucher(Customer, Voucher, Total).
```

Ada receives a voucher for a total of 15. Bea is blocked. `Price` and `Item` are local to the aggregate; `Customer` is its outer grouping parameter. Additional template keys keep equal prices for different items distinct. `count`, `sum`, `min`, and `max` operate on distinct template tuples; empty count and sum are zero, while empty minimum and maximum have no result.

`@fresh(Voucher)` creates a typed opaque witness belonging to that rule firing. It must be a rule's final goal and bind a new variable. Witnesses are deterministic within the program; they are not globally unique random identifiers. Source programs cannot supply witness constants.

```sh
./target/release/eyelight -c examples/orders.eye
```

This checks all nine conditions: C5 recomputes calculations and aggregates, C6 verifies complete lower-stratum boundaries and absence tests, and C8 verifies witness identity. Other conditions validate the rule applications, references, dependency order, relevance, and query isolation.

**Omitted facts cannot justify a negative test or a smaller aggregate.** The certificate includes the full relevant stratum prefix. The checker requires its input facts and verifies closure under every applicable source rule, without running the forward evaluator or adding missing conclusions. It shares the value model and pure matching, built-in, and aggregate semantics with the evaluator.

Variables in `not` goals must be bound earlier. Negative and aggregate dependencies must be stratified. Rules that produce values through built-ins, aggregation, or fresh witnesses cannot define recursive relations; this prevents unbounded value generation. Positive recursion with tests on bound values remains supported. The [language reference](https://eyereasoner.github.io/eyelight/SPEC/#4-rules-and-body-goals) gives the exact modes and restrictions.

## Use the command line

| Option | Result |
| --- | --- |
| No output option | Query answers; without queries, newly derived facts |
| `-e`, `--explain` | Answers and a numbered explanation |
| `-p`, `--proof` | A portable text certificate |
| `-c`, `--prove-and-check` | A report from checking a freshly generated certificate |
| `--check-proof FILE` | Check a saved certificate against the supplied source files |
| `--max-facts N` | Maximum stored fact/evidence steps; default 1,000,000 |
| `--max-matches N` | Maximum matching/work operations; default 10,000,000 |
| `-h`, `--help` | Usage |
| `-v`, `--version` | Installed version |

Choose one output option per invocation. Success returns status 0. Parse errors, resource limits, file errors, and invalid proofs return status 1 and write a specific diagnostic to stderr. Reaching a limit yields an incomplete error, with no partial stdout presented as complete answers. Limits count matching/work operations and stored fact/evidence steps; they are not a whole-process memory or time limit.

Files are parsed separately and merged in the supplied order. Variables belong to individual clauses and queries. Use `-` for stdin, once per invocation:

```sh
./target/release/eyelight rules.eye data.eye
./target/release/eyelight - < examples/socrates.eye
./target/release/eyelight --check-proof - examples/socrates.eye < /tmp/socrates.proof
```

Use the same source order when checking a saved certificate. Source-fact, rule, and query IDs are numbered separately from 1 in that order.

## Open the playground

The included `pkg/` directory contains the WebAssembly build. From this checkout:

```sh
python3 -m http.server 8000
```

Open <http://localhost:8000/playground.html>. Choose an example and press **Run**. The **Show** menu selects answers, an explanation, a certificate, or proof checking. To check a saved certificate, select **Check saved proof**, paste it below the editors, and keep its source program above.

The engine runs in a worker. **Stop** terminates that worker, including while WebAssembly is loading. You can run again immediately. Input and output have equal widths on larger screens and stack on phones. Ctrl+Enter or Command+Enter runs the program from the editor.

**Share link** uses `#example=socrates` for a loaded, unchanged example in Answers mode with no saved proof. Otherwise it puts the program, mode, and saved proof in the URL fragment. A recipient can read them. Creating the link uploads nothing; shared programs are loaded but never run automatically. Links containing large programs can be long. Inputs are autosaved locally when browser storage is available.

To rebuild the browser package, install `wasm-pack` and the Rust WebAssembly target:

```sh
rustup target add wasm32-unknown-unknown
./scripts/build-playground
```

## Embed the engine

```rust
use eyelight::{check, parse, reason, Limits};

fn main() -> Result<(), eyelight::Error> {
    let program = parse("human(socrates). mortal(X) :- human(X). ?- mortal(Who).")?;
    let run = reason(&program, Limits::default())?;
    print!("{}", run.output(&program));

    let certificate = run.certificate(&program);
    let report = check(&program, &certificate)?;
    assert!(report.valid());
    assert_eq!(report.claims, 1);
    Ok(())
}
```

`Run` exposes stored proof steps, projected query answers, and the number of attempted matches. `Certificate::parse` reads the certificate format. `execute` provides the same string interface used by the browser. Parser and library errors use `eyelight::Error`; incomplete evaluation returns an error rather than a successful `Run`.

`check_with_limits` accepts explicit evidence-size and matching limits; `check` uses defaults. Both return `Ok(report)` when semantic checking finishes, including when the proof is invalid. Inspect **`report.valid()`** before accepting it, or call `report.require_valid()?` to turn an invalid report into an error. `report.conditions` lists all nine conditions with counts and applicability; `report.failures` supplies their diagnostics. `report.condition(eyelight::Condition::C9)` selects one condition. Unreadable certificates and malformed source programs return errors before a report can be produced.

## Current scope

Implemented: positive recursion, indexed joins and an agenda, explicit arithmetic and string built-ins, stratified negation, grouped set aggregation, scoped fresh witnesses, query projection, structural value identity, explanations, certificate generation, all C1–C9 checks for their corresponding features, a native CLI, and a browser worker.

Not implemented: floating-point arithmetic, lists or general compound terms, inverse built-in solving, nested aggregates, recursive value generation, implicit existential heads, quoted graphs, generated rules, N3 parsing, general Prolog execution, or demand-driven tabling. Quoted IRIs in `triple(S, P, O)` are ordinary atom values, without additional RDF entailment built in.

The safety and recursion restrictions give finite strata. Actual completion remains subject to resource limits, available memory, and cancellation. Closed-boundary proofs can be substantially larger than proofs using positive rules alone because they carry the completed stratum prefix.

## Develop and verify

The development checks use Rust, Cargo, and Node.js 22 or later. CI uses Node.js 24.

```sh
./scripts/test-all
cargo clippy --target wasm32-unknown-unknown --lib -- -D warnings
./scripts/build-playground
```

Tests cover language behavior, the CLI, forged certificates for every applicable proof condition, explicit per-program applicability, built-ins, aggregates, witnesses, and the worker bridge. Reachability, stratified exclusions, and aggregate counts are checked against independent graph algorithms on all 512 directed graphs over three nodes. Browser checks exercise actual WebAssembly execution, C1–C9 reports, forged-proof rejection, cancellation, sharing, and narrow-screen layout.

`tests/browser.mjs` automates those browser checks through Chrome's local debugging interface. With Node.js 22 or later, start the server and Chrome in separate terminals, then run the checks:

```sh
python3 -m http.server 8766 --bind 127.0.0.1
google-chrome --headless --remote-debugging-port=9226 --user-data-dir=/tmp/eyelight-browser about:blank
node tests/browser.mjs
```

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

The script switches to `main`, pulls with `--ff-only`, bumps `Cargo.toml` with `cargo set-version`, and synchronizes the guide, specification, and playground version labels. It rebuilds WebAssembly, runs `./scripts/test-all` and Wasm clippy, then commits **all working-tree changes**, creates an annotated `v<version>` tag, pushes the branch and tag together, and creates the GitHub release. Review `git status` and `git diff` first. The push also starts CI and the Pages deployment.

A failed build or check stops before committing or publishing, leaving the local version changes available for inspection. Resolve the failure and finish that version rather than bumping again. If a commit and tag were created but publication stopped, retry only the unfinished steps, substituting the actual tag:

```sh
git push --atomic origin main v0.1.1
git tag --list v0.1.1 --format='%(contents)' |
  gh release create v0.1.1 --title=v0.1.1 --notes-file -
```

Skip the push if it already succeeded, and create the GitHub release only if it is still missing. The release contains GitHub's source archives. A separate tag workflow is not needed.

## Publish the website

The **Deploy GitHub Pages** workflow builds the site on pushes to `main` and can also be started manually. Set the repository's Pages source to **GitHub Actions**. The workflow checks the website and committed WebAssembly package, renders the guide and specification with Jekyll, and deploys the resulting static artifact.

Open the published [guide](https://eyereasoner.github.io/eyelight/), [specification](https://eyereasoner.github.io/eyelight/SPEC/), or [playground](https://eyereasoner.github.io/eyelight/playground/). Documentation links in this repository point to those rendered pages. The `playground.html` route remains available too.

`node scripts/prepare-pages.mjs` prepares `_pages/` for the Jekyll build; `_site/` holds the rendered artifact. Both directories are ignored by Git. The Pages action supplies `PAGES_BASE_PATH`, so local links and assets work beneath `/eyelight/` or a different configured prefix. The site uses the committed `pkg/`; changing the package version requires rebuilding it before publishing. The deployment check rejects a browser package whose exported version differs from `Cargo.toml`.

| File | Responsibility |
| --- | --- |
| `src/model.rs` | Values, facts, rules, queries, and validation |
| `src/parser.rs` | The `.eye` language |
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

MIT licensed. See [LICENSE](https://github.com/eyereasoner/eyelight/blob/main/LICENSE).
