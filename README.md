<img src="assets/eyelight.svg" alt="" width="80" height="80" align="left">

# eyelight

A small relational reasoner with explanations and independently checked proof certificates.

<br clear="left">

**Version 0.1.0** · [Guide](https://eyereasoner.github.io/eyelight/) · [Language](https://eyereasoner.github.io/eyelight/SPEC/) · [Playground](https://eyereasoner.github.io/eyelight/playground/) · [Examples](https://github.com/eyereasoner/eyelight/tree/main/examples/)

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

Eyelight's `.eye` language uses familiar clause notation. It implements **finite positive relational rules**, a deliberately small Datalog-style language. It does not parse N3 or implement general Prolog. The [specification](https://eyereasoner.github.io/eyelight/SPEC/) describes the complete first-version language.

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
  C5 re-decision: not applicable (no interpreted built-ins)
  C6 boundary consistency: not applicable (no trust obligations or inference fuses)
  C7 relevance: 2 checked, 0 failed
  C8 witness integrity: not applicable (no existential witnesses)
  C9 query isolation: 1 checked, 0 failed
Source assertions are assumed. Answer completeness is not checked.
```

A valid certificate establishes **positive support for its claims**. It assumes the source assertions, does not authenticate their authors, and does not establish that every answer was found. A failed query prints `false.` but has no positive claim to certify. A certificate with zero claims proves no answers and no absence. The checker shares the term representation and closed matching primitives with the evaluator; it has a separate verification loop and performs no inference search.

The [C1–C9 contract](https://eyereasoner.github.io/eyelight/SPEC/#8-verification) defines the requirements and counting units. Invalid reports identify the failing condition and affected step or claim. C5, C6, and C8 are explicitly not applicable to the current language: it has no interpreted built-ins, trust obligations, inference fuses, or existential generation. Certificates cannot add those features through unsupported metadata.

## Use the command line

| Option | Result |
| --- | --- |
| No output option | Query answers; without queries, newly derived facts |
| `-e`, `--explain` | Answers and a numbered explanation |
| `-p`, `--proof` | A portable text certificate |
| `-c`, `--prove-and-check` | A report from checking a freshly generated certificate |
| `--check-proof FILE` | Check a saved certificate against the supplied source files |
| `--max-facts N` | Maximum distinct stored facts; default 1,000,000 |
| `--max-matches N` | Maximum attempted candidate matches; default 10,000,000 |
| `-h`, `--help` | Usage |
| `-v`, `--version` | Installed version |

Choose one output option per invocation. Success returns status 0. Parse errors, resource limits, file errors, and invalid proofs return status 1 and write a specific diagnostic to stderr. Reaching a limit yields an incomplete error, with no partial stdout presented as complete answers. Limits count operations and facts; they are not a whole-process memory or time limit.

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

`check` returns `Ok(report)` when semantic checking finishes, including when the proof is invalid. Inspect **`report.valid()`** before accepting it, or call `report.require_valid()?` to turn an invalid report into an error. `report.conditions` lists all nine conditions with counts and applicability; `report.failures` supplies their diagnostics. `report.condition(eyelight::Condition::C9)` selects one condition. Unreadable certificates and malformed source programs return errors before a report can be produced.

## Scope of this first version

Implemented: positive recursion, joins, query projection, structural value identity, indexed fact lookup, an agenda, numbered explanations, certificate generation and C1–C9 checking reports for the supported language, a native CLI, and a browser worker.

Not implemented: arithmetic, lists or compound terms, negation, aggregation, existential heads, quoted graphs, generated rules, N3 parsing, general Prolog execution, or demand-driven tabling. Quoted IRIs in `triple(S, P, O)` are ordinary atom values, without additional RDF entailment built in.

The finite value domain and safe heads guarantee a finite positive closure. Actual completion remains subject to resource limits, available memory, and cancellation. No speed or feature-parity claim against other engines is made in this release.

## Develop and verify

The development checks use Rust, Cargo, and Node.js 22 or later. CI uses Node.js 24.

```sh
./scripts/test-all
cargo clippy --target wasm32-unknown-unknown --lib -- -D warnings
./scripts/build-playground
```

Tests cover language behavior, the CLI, forged certificates for every applicable proof condition, explicit non-applicability, and the worker bridge. Reachability is also checked against an independent graph algorithm on all 512 directed graphs over three nodes. Browser checks exercise actual WebAssembly execution, C1–C9 reports, forged-proof rejection, cancellation, sharing, and narrow-screen layout.

`tests/browser.mjs` automates those browser checks through Chrome's local debugging interface. With Node.js 22 or later, start the server and Chrome in separate terminals, then run the checks:

```sh
python3 -m http.server 8766 --bind 127.0.0.1
google-chrome --headless --remote-debugging-port=9226 --user-data-dir=/tmp/eyelight-browser about:blank
node tests/browser.mjs
```

## Publish the website

The **Deploy GitHub Pages** workflow builds the site on pushes to `main` and can also be started manually. Set the repository's Pages source to **GitHub Actions**. The workflow checks the website and committed WebAssembly package, renders the guide and specification with Jekyll, and deploys the resulting static artifact.

Open the published [guide](https://eyereasoner.github.io/eyelight/), [specification](https://eyereasoner.github.io/eyelight/SPEC/), or [playground](https://eyereasoner.github.io/eyelight/playground/). Documentation links in this repository point to those rendered pages. The `playground.html` route remains available too.

`node scripts/prepare-pages.mjs` prepares `_pages/` for the Jekyll build; `_site/` holds the rendered artifact. Both directories are ignored by Git. The Pages action supplies `PAGES_BASE_PATH`, so local links and assets work beneath `/eyelight/` or a different configured prefix. The site uses the committed `pkg/`; changing the package version requires rebuilding it before publishing. The deployment check rejects a browser package whose exported version differs from `Cargo.toml`.

| File | Responsibility |
| --- | --- |
| `src/model.rs` | Values, facts, rules, queries, and validation |
| `src/parser.rs` | The `.eye` language |
| `src/engine.rs` | Indexed joins, agenda, limits, and answer rendering |
| `src/proof.rs` | Certificates and the separate checker |
| `src/main.rs` | Native command line and file I/O |
| `src/wasm.rs` | Browser bindings |
| `playground*.js` | UI and cancellable worker |
| `scripts/prepare-pages.mjs`, `scripts/pages-layout.html` | Pages staging and documentation layout |
| `.github/workflows/pages.yml` | Website build and deployment |

MIT licensed. See [LICENSE](https://github.com/eyereasoner/eyelight/blob/main/LICENSE).
