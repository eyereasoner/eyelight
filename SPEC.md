# Eyelight language and proof contract

Version **0.1.0**. This document specifies the implemented initial language. The [guide](https://eyereasoner.github.io/eyelight/) provides runnable examples.

## 1. Program

A program consists of ground facts, rules, and queries. All statements end in a period. Whitespace separates tokens but is otherwise insignificant. `%` and `#` introduce comments through the end of the line outside quotes.

```prolog
human(socrates).
mortal(Person) :- human(Person).
?- mortal(Who).
```

The parser reads the whole program before evaluation. Facts and rules appearing after a query are available to that query. Local files supplied together form one program; each file must contain complete statements.

## 2. Names and values

Bare names match `[a-z][A-Za-z0-9_]*`. A single-quoted name is an atom with exactly the enclosed text; `socrates` and `'socrates'` denote the same atom. Predicates use these same name forms and are identified by name and arity.

Argument values are atoms, double-quoted strings, or signed decimal 64-bit integers. Strings and atoms are distinct. `42`, `'42'`, and `"42"` are three distinct values. Leading zeros and negative zero normalize as integer values. Integer overflow is a parse error. Floating-point literals are unsupported.

Quoted values support literal Unicode and the escapes `\n`, `\r`, `\t`, `\\`, `\'`, and `\"`. Quotes are escaped with a backslash; doubling a quote is not an escape. No Unicode normalization is performed. An IRI can be written as a quoted atom; it has no implicit dereferencing or interpretation.

## 3. Variables and relations

Variables match `[A-Z_][A-Za-z0-9_]*`. Their scope is one rule or query. Repeated named variables must match the same value. Every occurrence of `_` is fresh and omitted from query output. Other names, including `_Named`, remain shared and are displayed.

```prolog
parent(ada, bea).
ready.
?- parent(Who, _).
```

A relation has zero to 64 arguments. Write zero-arity relations without parentheses: `ready`, not `ready()`. Values cannot contain variables or compound terms; a variable stands for one complete value. Predicate variables are unsupported.

## 4. Rules

A rule has one head and 1 to 64 body goals, separated by commas. Every head variable must occur in a positive body goal. Facts cannot contain variables. Rule bodies are conjunctions and alternative rules express alternative derivations.

```prolog
grandparent(Older, Younger) :-
    parent(Older, Middle),
    parent(Middle, Younger).
```

All rule variables are universally quantified over the program's finite set of constant values. Values are structurally equal only when their kinds and contents agree. Matching binds the rule's variables against closed stored facts. There are no implicit builtin predicate names; a name such as `sum` or `not` is an ordinary relation and has no additional behavior.

## 5. Meaning and execution

Facts have set semantics. A positive program denotes its least closure: the smallest fact set containing all input facts and closed under every rule application. A repeated fact or additional derivation does not add another tuple. Different predicate arities denote different relations.

The implementation uses an agenda of newly inserted facts, indexes by relation and argument value, and joins matching premises. It stores the first successful justification for a fact. Source facts are inserted before rule execution. A stored rule step therefore refers only to earlier stored facts. A cycle cannot supply its own unsupported evidence.

The language cannot construct new values. Safe heads therefore give a finite closure, including when the input contains positive recursive cycles. This is not a guarantee that arbitrary programs finish within available time or memory. Default limits permit 1,000,000 distinct facts and 10,000,000 attempted candidate matches, including query matching. Reaching a limit is an error and produces no successful complete result. Output order and the choice among successful justifications are unspecified.

## 6. Queries and output

A query is a conjunction of 1 to 64 goals. Queries run over the completed closure and do not add facts. Results project named variables in first-occurrence order and remove duplicate projected tuples. A successful query with no displayed variables prints `true.` once. A query with no solutions prints `false.`. This describes failure to find a match in the completed program, not an asserted negative fact.

With queries, ordinary output contains their headings and answers in query order. Without queries, it contains facts newly derived by rules and omits facts already present in the input. A failed or limited run does not print partial answers as complete output.

## 7. Proof certificates

Certificates are UTF-8 text beginning with the exact line `eyelight-proof 1`. Blank subsequent lines are permitted. Step IDs are contiguous positive integers starting at 1, followed by claim records. Steps cannot appear after claims. Examples:

```text
eyelight-proof 1
step 1 source 1 : human(socrates).
step 2 rule 1 [1] : mortal(socrates).
answer 1 [2]
```

Source fact IDs, rule IDs, and query IDs refer to their separate lists in source order. A `source` step must equal the cited ground source fact. A `rule` step lists one earlier step ID for each body goal, in body order. A fact may support more than one goal, so premise IDs may repeat. Recorded conclusions must be ground.

An `answer QUERY [PREMISES]` claim supplies one fact per query goal in order. It establishes the projected tuple obtained from one consistent match. With no source queries, `derived STEP` instead claims a rule-derived conclusion. A certificate for a queried program cannot use `derived` claims. Query answers are selections, never additional inference steps.

The generator keeps the selected claims and their transitive dependencies, renumbering the retained steps. It omits unrelated facts and derivations.

## 8. Verification

The separate checker implements this C1–C9 contract for the current language:

| Condition | Requirement and current applicability |
| --- | --- |
| **C1 · Resolution** | Each source step equals its cited ground input fact. Each rule step supplies every body premise in order, matches them under one consistent substitution, and states exactly the instantiated head of the cited source rule. Matching binds only rule variables; recorded facts are closed. |
| **C2 · Well-foundedness** | Every rule premise refers to an earlier step. Cycles, self-support, and acyclic forward references are invalid; this certificate format requires topological order. |
| **C3 · Justification** | Each step has exactly one well-formed supported justification: source or rule. Recorded conclusions are ground and unique. Answer claims have unique projected tuples per query; derived claims have unique step IDs. Unsupported metadata and malformed records are rejected by the reader with a C3-labelled error. |
| **C4 · Coverage** | Every source-fact, rule, query, premise, and claimed-step ID resolves in the supplied program or certificate. Query claims provide evidence for all their goals, as checked by C9. |
| **C5 · Re-decision** | Not applicable: the language has no interpreted built-ins. An ordinary relation named `sum`, for example, has no arithmetic interpretation. A certificate cannot introduce a builtin justification. |
| **C6 · Boundary consistency** | Not applicable: the language has no trust obligations or inference fuses. Certificates cannot introduce trusted, negative, or collection assumptions. Source assertions remain the fixed premises of checking. |
| **C7 · Relevance** | Every recorded step contributes through premise dependencies to a claim. A `derived` claim must cite a rule conclusion absent from the source facts, matching ordinary output's exclusion of inputs. A query may select a supported input fact. |
| **C8 · Witness integrity** | Not applicable: the language has no existential generation or witness terms. Unsafe heads are rejected when validating the source program. A certificate cannot introduce witness justifications. |
| **C9 · Query isolation** | Query answers are selections, never inference steps or premise providers. Every answer records a complete source query application with consistent bindings across every goal. When queries are present, derived-step claims are forbidden. |

The report lists all nine conditions in order. Applicable conditions show attempted check and failure counts. C5, C6, and C8 show **not applicable** with a reason, with zero checks and failures. They are not reported as successful verification of unsupported features. Introducing the corresponding language features requires implementing their verification and revising applicability.

Counting units are source/rule steps for C1; rule dependency lists for C2; steps and claims for C3; individual references for C4; steps and resolved derived claims for C7; and query applications or derived-claim mode checks for C9. Checks whose required references do not resolve are skipped, with the missing evidence reported under C4. Counts describe local checks, not a count of established conclusions. Relevance walks resolved dependency edges even when other conditions fail.

A semantic failure produces an **invalid** report with a condition, 1-based step or claim location, and a specific diagnostic. The checker collects independently decidable failures. A report is **checked** exactly when no failures remain. Malformed source programs and unreadable certificates return errors before a semantic report can be constructed. CLI and browser checking fail on invalid reports; CLI failures return status 1, write the report to stderr, and emit no successful stdout.

Library `check` returns `Result<Report, Error>`: `Ok(report)` means checking finished, and callers must inspect `report.valid()` or call `report.require_valid()`. `Report::condition(Condition::C1)` and the ordered `conditions` array expose counts and applicability; `failures` exposes condition-labelled diagnostics.

The checker does not call the evaluator, enumerate missing answers, or accept trusted builtin steps. It shares the value model and closed matching primitives with the evaluator. Source assertions remain assumptions. Acceptance establishes support for the recorded positive claims, not completeness, absence, source authenticity, or real-world truth. The empty certificate passes with zero claims, zero attempted checks, and no additional guarantees.

## 9. Explicit boundaries

This version has no arithmetic evaluation, constructors, lists, functions, negation, aggregation, existential generation, quoted graphs, dynamic rules, procedural side effects, or general Prolog control. It has no N3, Turtle, or other RDF syntax parser. RDF-shaped data can be modeled in an ordinary relation without changing this contract.
