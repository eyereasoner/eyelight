# Eyelight language and proof contract

Version **0.1.7**. This document specifies the implemented language. The [guide](https://eyereasoner.github.io/eyelight/) provides runnable examples.

## 1. Program

Programs use the `.pl` extension and Prolog-style clause syntax. A program consists of ground facts, rules, and queries. Statements end in a period. Whitespace separates tokens but is otherwise insignificant. `%` and `#` introduce comments through the end of the line outside quotes.

```prolog
human(socrates).
mortal(Person) :- human(Person).
?- mortal(Who).
```

The parser reads the whole program before evaluation. Facts and rules appearing after a query are available to it. Local files supplied together form one program; each file must contain complete statements. The merged program is validated again before reasoning or checking.

## 2. Names and values

Bare names match `[a-z][A-Za-z0-9_]*`. A single-quoted name is an atom with exactly the enclosed text; `socrates` and `'socrates'` denote the same atom. Predicates use these name forms and are identified by name and arity.

Source argument values are atoms, double-quoted strings, or signed decimal 64-bit integers. Strings and atoms are distinct. `42`, `'42'`, and `"42"` are three distinct values. Leading zeros and negative zero normalize as integers. Integer overflow in a literal is a parse error. Floating-point literals are unsupported.

Quoted values support literal Unicode and the escapes `\n`, `\r`, `\t`, `\\`, `\'`, and `\"`. Quotes are escaped with a backslash; doubling a quote is not an escape. No Unicode normalization is performed. An IRI written as a quoted atom has no implicit dereferencing or interpretation.

Evaluation can also produce **opaque witness values**. Their printed form is `~"identity"`; the reader accepts this form in certificates, but source programs cannot supply witness constants. A witness is distinct from an atom or string with the same text.

## 3. Variables and relations

Variables match `[A-Z_][A-Za-z0-9_]*`. Their scope is one rule or query, except for aggregate-local variables described below. Repeated named variables must match the same value. Every occurrence of `_` is fresh and omitted from query output. Other names, including `_Named`, remain shared and are displayed.

```prolog
parent(ada, bea).
ready.
?- parent(Who, _).
```

A relation has zero to 64 arguments. Write zero-arity relations without parentheses: `ready`, not `ready()`. There are no compound terms, predicate variables, or implicit interpretations of ordinary predicate names. `add`, `sum`, and `not` remain ordinary names when used as relations. Interpreted relations have an explicit `@` prefix.

## 4. Rules and body goals

A rule has one relational head and 1 to 64 body goals, separated by commas. Facts cannot contain variables. Every head variable must be bound by the body. Body goals are processed left to right, using structural value identity, without implicit conversions. Matching binds only the goal's variables against closed values. Alternative rules express alternative derivations.

```prolog
grandparent(Older, Younger) :-
    parent(Older, Middle),
    parent(Middle, Younger).
```

An ordinary positive relation can bind its variables. The other goal forms have the safety requirements below. Invalid modes are rejected before evaluation, even if the unsafe rule would never fire.

### 4.1 Built-ins as interpreted relations

Built-ins describe potentially infinite sets of true tuples. Eyelight decides individual tuples or computes a deterministic output from bound inputs; it never enumerates an unbounded domain.

```prolog
line(book, 12, 2).
total(Item, Total) :- line(Item, Price, Quantity), @mul(Price, Quantity, Total).
?- total(Item, Total), @ge(Total, 20).
?- @add(2, 3, Result).
```

| Built-in | Required inputs | Meaning |
| --- | --- | --- |
| `@add(A, B, C)` | A, B | Integer C = A + B |
| `@sub(A, B, C)` | A, B | Integer C = A − B |
| `@mul(A, B, C)` | A, B | Integer C = A × B |
| `@div(A, B, C)` | A, B | Integer division, truncated toward zero |
| `@mod(A, B, C)` | A, B | Remainder with the dividend's sign |
| `@abs(A, B)` | A | Integer absolute value |
| `@eq(A, B)`, `@neq(A, B)` | A, B | Structural equality or inequality |
| `@lt(A, B)`, `@le(A, B)`, `@gt(A, B)`, `@ge(A, B)` | A, B | Integer comparisons |
| `@concat(A, B, C)` | A, B | Concatenation of two strings |
| `@strlen(A, N)` | A | String length in Unicode scalar values |

Each required input must be a constant or a variable bound by earlier goals. An output can bind a new variable or test an existing value. A wrong type, false comparison, division by zero, or overflowing arithmetic tuple has no solution. Undefined built-in tuples do not wrap or produce partial numeric results. Unknown built-ins and wrong arities are errors.

For example, `@add(2,3,5)` succeeds and `@add(2,3,6)` fails. `@add(X,3,5)` is rejected for an unbound input; inverse arithmetic solving is not implemented.

### 4.2 Stratified negation

`not Relation(...)` tests absence from a completed lower stratum. Every variable in a negated goal must already be bound; negation never binds a variable.

```prolog
eligible(Person) :- registered(Person), not blocked(Person).
```

This is closed-world negation relative to the supplied program. It is distinct from a claim about an external world. A relation may not depend negatively, directly or indirectly, on itself. Positive recursion remains supported.

### 4.3 Aggregation

```prolog
budget(Department, Count, Total) :-
    department(Department),
    count { Item : purchase(Department, Item, Price) } = Count,
    sum { Price, Item : purchase(Department, Item, Price) } = Total.
```

The goal form is `Operation { Template : Goals } = Result`, with `count`, `sum`, `min`, or `max`. The template contains 1 to 64 terms; the inner conjunction contains 1 to 64 goals. Each template variable must be bound by that conjunction or by earlier outer goals.

Names already bound outside are grouping parameters. Other names inside braces are local to that aggregate and are neither exported nor projected by the enclosing query. Only `Result` can introduce an outer binding. A result already bound outside is tested for equality. Later outer goals cannot capture an aggregate's local variables.

Aggregation operates on a **set of distinct complete template tuples**. `count` counts these tuples. `sum`, `min`, and `max` use their first element, which must be an integer. Additional elements are keys: `sum { Price, Item : ... }` counts two equal prices on different items, whereas `sum { Price : ... }` counts each distinct price once. Repeated source facts never create bag duplicates.

The count and sum of an empty set are zero. Empty minimum and maximum have no solution. A noninteger numeric aggregate or an overflowing sum is an evaluation error; it cannot produce a successful partial answer. Count must also fit a signed 64-bit integer.

Inner goals can be positive relations, built-ins, or safe negation. Nested aggregates and fresh witnesses inside aggregates are rejected; use an intermediate relation instead. All relational dependencies inside an aggregate must belong to strictly lower strata than its rule's head.

### 4.4 Fresh witnesses

```prolog
ticket(Person, Ticket) :- registered(Person), @fresh(Ticket).
```

`@fresh(W)` is allowed only as the final goal of a rule. W must be a new variable. It creates a typed opaque witness for that rule firing: the rule ID, goal position, and complete preceding variable substitution determine its identity. Different firings or source rules receive distinct identities. Repeating the same firing reuses the same identity, preserving set semantics.

The identity is deterministic within the supplied program, not a globally unique random identifier across programs or runs. Source facts cannot preassign witness identities. Other rules can carry a witness as an ordinary opaque value. Query goals and aggregate bodies cannot create witnesses. General implicit existential heads are unsupported; generation must be explicit.

## 5. Meaning and execution

Facts have set semantics. Ordinary positive rules denote their least closure. A repeated fact or additional derivation does not add another tuple. Source facts are inserted before rule execution; the first successful justification for each new fact is retained.

A relation's stratum is at least that of every positive dependency and strictly greater than every negative or aggregate dependency. An aggregate rule has stratum at least one even when its inner goals use only built-ins. The implementation rejects dependency cycles containing a strict edge, and finishes each stratum's positive fixed point before evaluating the next. Queries run after all strata finish.

Within a stratum, an agenda and indexes by relation and argument value drive joins. Built-in decisions, negative tests, aggregates, and fresh-witness records are separate proof evidence; they never enter ordinary relation indexes. Fully ground positive body goals are dispatched by their complete tuple; other goals with constants use one fixed argument, and all-variable goals use their relation. Matching still checks every argument and shared binding. The packaged deep taxonomies through 100,000 levels have expected linear dispatch and matching work in their depth. This does not bound arbitrary joins by their input size.

Value-producing built-ins, aggregates, and `@fresh` cannot occur in rules defining a recursive relation, including indirect recursion. This deliberately conservative restriction prevents recursive value generation. Comparisons and other tests on already bound values remain usable in positive recursion. Together with finite source data, this gives finite strata and a finite closure. General recursive arithmetic and unrestricted existential generation are unsupported.

Completion remains subject to available time and memory. Default limits permit 1,000,000 stored fact/evidence steps and 10,000,000 matching/work operations. Snapshot preparation and aggregate matching count toward the work limit. Limits are errors, with no successful partial answers. Output order and the choice among successful justifications are unspecified.

## 6. Queries and output

A query is a conjunction of 1 to 64 body goals, with the same safety requirements as rules, except that it cannot create fresh witnesses. Queries run over the completed strata and never add relational facts. Results project named outer variables in first-occurrence order and remove duplicate projected tuples. Aggregate-local variables are omitted.

A successful query with no displayed variables prints `true.` once. A query with no solutions prints `false.`. The latter is an output convention, not a separately certified assertion of absence. An explicit safe `not` goal can instead supply checked closed-world evidence in a successful answer.

With queries, ordinary output contains headings and answers in query order. Without queries, it contains newly derived relational facts and omits source facts and auxiliary proof evidence. A failed or limited run does not print partial answers as complete output.

## 7. Proof certificates

Certificates begin with the exact line `eyelight-proof 1`. Blank subsequent lines are permitted. Step IDs are contiguous positive integers starting at 1, followed by claims. Steps and boundary records cannot appear after claims.

```text
eyelight-proof 1
step 1 source 1 : human(socrates).
step 2 rule 1 [1] : mortal(socrates).
answer 1 [2]
```

Source-fact, rule, and query IDs refer to separate lists in source order. Every recorded atom is ground. A `rule RULE [PREMISES]` step lists one earlier evidence step per body goal, in body order; IDs may repeat. The checker matches both terms and evidence kinds. A negative or built-in decision cannot masquerade as a positive relational premise.

Supported step justifications are:

| Record after `step ID` | Evidence |
| --- | --- |
| `source FACT : ATOM.` | The cited ground input fact |
| `rule RULE [PREMISES] : ATOM.` | A complete source-rule application |
| `builtin : ATOM.` | A closed interpreted relation tuple, recomputed by the checker |
| `absent BOUNDARY : ATOM.` | Absence of the closed relational atom from a certified boundary |
| `aggregate OWNER POSITION BOUNDARY : ATOM.` | Recomputed source aggregate, with outer bindings and result |
| `fresh RULE POSITION : ATOM.` | A witness and its source firing context |

An aggregate owner is `r<RULE>` or `q<QUERY>`, and positions are 1-based body-goal positions. Aggregate evidence uses synthetic predicates `$aggregate:OWNER:POSITION`; witness evidence uses `$fresh:rRULE:POSITION`. Their arguments are preceding variables in sorted name order, followed by the result. These records are auxiliary evidence, not ordinary program facts.

`boundary ID level LEVEL [STEPS]` lists a finite model containing every relational fact in the completed stratum prefix through LEVEL. Negative decisions cite exactly their relation's stratum. Aggregate decisions cite the maximum stratum of their inner relations (zero when there are none). Boundary IDs are contiguous from one; levels are nonnegative and unique. Every member must precede a decision using that boundary.

The checker verifies each boundary includes the prefix's source facts and is closed under every source rule whose head belongs to the prefix. It matches each such rule against the supplied model and requires its head to be present; it does not add or derive missing facts. Recorded well-founded justifications establish support for model members. Support plus closure establishes the completed prefix, so omitting a derived counterexample or aggregate member cannot validate a false result.

An `answer QUERY [PREMISES]` claim supplies complete evidence for its source query. With no queries, `derived STEP` instead claims a new rule conclusion. A queried program cannot use `derived` claims. Query answers are selections, never inference steps or premise providers. Query-owned aggregate evidence cannot satisfy a rule-owned aggregate goal.

The generator keeps selected claims, their transitive dependencies, and complete boundaries they require, then renumbers them. It marks dependencies iteratively and scans retained IDs in order, so a deep proof chain does not require recursive traversal or sorting. Boundaries can make proofs considerably larger than proofs of positive rules alone: completeness requires all members of the relevant stratum prefix.

## 8. Verification

The checker implements the following C1–C9 contract:

| Condition | Requirement |
| --- | --- |
| **C1 · Resolution** | Each source step equals its cited input fact. Each rule step matches every body goal with the correct evidence kind under one consistent substitution and states exactly its instantiated head. Matching binds only rule variables. |
| **C2 · Well-foundedness** | Rule premises and boundary members precede the decisions that use them. Cycles, self-support, and forward references are invalid. Stratum validation also forbids negative or aggregate dependency cycles. |
| **C3 · Justification** | Every record has one supported justification and a ground conclusion. Relational conclusions, identical decisions, boundary levels, derived claims, and projected query answers are unique. Unsupported metadata and malformed records are rejected by the reader with a C3-labelled error. |
| **C4 · Coverage** | Every source fact, rule, query, owner, goal position, boundary, and cited step resolves in the supplied program or certificate. Complete body and query coverage is checked by C1 and C9. |
| **C5 · Re-decision** | Every recorded built-in tuple is decided again with closed arguments. Every aggregate is recomputed from its source definition, grouping bindings, and certified boundary, including distinct tuple keys, types, and empty-set behavior. |
| **C6 · Boundary consistency** | Boundaries contain only relational facts at or below their level, include all corresponding source facts, and are closed under all corresponding rules. Negative decisions must not be contradicted by the completed boundary. Negative and aggregate decisions must cite the exact dependency level. |
| **C7 · Relevance** | Every step and boundary contributes to a claim through ordinary or required boundary dependencies. A `derived` claim cites a rule conclusion absent from source facts. A query may select a supported input fact. |
| **C8 · Witness integrity** | Every fresh decision names a fresh source goal and carries exactly the typed witness determined by that source-rule firing. A witness from another firing cannot be substituted, reused, or supplied as a source assertion. C1 checks the context when the decision is consumed. |
| **C9 · Query isolation** | Query answers are selections, never inference steps. Each answer matches every source-query goal with the correct evidence kind and consistent bindings. Derived claims are forbidden when queries are present. Query aggregate evidence cannot become rule evidence. |

Reports list all nine conditions in order. C5 is applicable when a program uses built-ins or aggregates; C6 when it uses negation or aggregates; C8 when it uses `@fresh`. A program without the corresponding feature reports **not applicable** with zero checks and failures. The [voucher example](https://github.com/eyereasoner/eyelight/blob/main/examples/orders.pl) performs nonzero successful checks under every condition.

Counts describe attempted local checks, not a count of established conclusions. C1 counts source/rule steps; C2 rule and boundary dependency lists; C3 steps, claims, and boundaries; C4 individual references; C5 decisions; C6 boundary models and decisions; C7 steps, boundaries, and resolved derived claims; C8 fresh decisions; C9 query applications or derived-claim mode checks. Checks requiring unresolved references are skipped, with the missing evidence reported under C4.

Semantic failures produce an **invalid** report with the condition, 1-based step/claim/boundary location, and diagnostic. Independently decidable failures are collected. A report is **checked** exactly when no failures remain. Invalid source programs, unreadable certificates, resource exhaustion, and arithmetic errors during boundary recomputation return errors. CLI and browser checking reject invalid reports; CLI failures return status 1 and no successful stdout.

Library `check` uses default limits. `check_with_limits` accepts an explicit `Limits` value for evidence size and matching work. Both return `Result<Report, Error>`: `Ok(report)` means checking finished, and callers must inspect `report.valid()` or use `report.require_valid()`. Ordered condition reports expose applicability and attempted/failed counts; failures expose their locations.

The checker never calls the evaluator or repairs a missing model. It shares term representation, matching primitives, and pure built-in/aggregate semantics with the evaluator; it is not a separately implemented arithmetic or term system. Source assertions remain assumptions. Acceptance establishes support for recorded claims, including explicit closed-world tests within certified strata. It does not establish answer completeness, source authenticity, or external-world truth. An empty certificate passes with zero claims and establishes nothing further.

## 9. Explicit boundaries

The implementation has no floating-point arithmetic, lists, general compound terms, inverse built-in solving, nested aggregates, unrestricted recursive value generation, implicit existential heads, quoted graphs, dynamic rules, procedural side effects, or general Prolog control. It has no N3, Turtle, or other RDF syntax parser. RDF-shaped data can be represented in an ordinary relation without changing this contract.
