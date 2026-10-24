use std::collections::BTreeSet;
use whye::{check, parse, reason, Certificate, Condition, Limits, Value};

fn run(source: &str) -> (whye::Program, whye::Run) {
    let program = parse(source).unwrap();
    let run = reason(&program, Limits::default()).unwrap();
    let proof = run.certificate(&program);
    assert_eq!(Certificate::parse(&proof.to_string()).unwrap(), proof);
    let report = check(&program, &proof).unwrap();
    assert!(report.valid(), "{report}\n{proof}");
    (program, run)
}
fn answers(source: &str) -> Vec<Vec<Vec<Value>>> {
    run(source)
        .1
        .answers
        .into_iter()
        .map(|a| a.into_iter().map(|a| a.values).collect())
        .collect()
}

#[test]
fn builtin_relations_calculate_and_check_closed_tuples() {
    for (call, expected) in [
        ("@add(2,3,X)", 5),
        ("@sub(2,3,X)", -1),
        ("@mul(-2,3,X)", -6),
        ("@div(-7,3,X)", -2),
        ("@mod(-7,3,X)", -1),
        ("@abs(-7,X)", 7),
        ("@strlen(\"é🙂\",X)", 2),
    ] {
        assert_eq!(
            answers(&format!("?- {call}.")),
            vec![vec![vec![Value::Integer(expected)]]]
        );
    }
    assert_eq!(
        answers("?- @concat(\"hello\",\" world\",X)."),
        vec![vec![vec![Value::Text("hello world".into())]]]
    );
    for call in [
        "@eq(a,a)",
        "@neq(1,'1')",
        "@lt(-1,0)",
        "@le(2,2)",
        "@gt(3,2)",
        "@ge(3,3)",
        "@add(2,3,5)",
    ] {
        assert_eq!(
            answers(&format!("?- {call}.")),
            vec![vec![vec![]]],
            "{call}"
        );
    }
}

#[test]
fn undefined_builtin_tuples_are_false_and_never_overflow() {
    for call in [
        "@add(9223372036854775807,1,X)",
        "@mul(9223372036854775807,2,X)",
        "@abs(-9223372036854775808,X)",
        "@div(1,0,X)",
        "@div(-9223372036854775808,-1,X)",
        "@mod(1,0,X)",
        "@add(\"1\",2,X)",
        "@lt(a,b)",
        "@eq(1,'1')",
        "@add(2,3,6)",
    ] {
        assert!(answers(&format!("?- {call}."))[0].is_empty(), "{call}");
    }
}

#[test]
fn ordinary_predicate_names_keep_their_meaning() {
    assert_eq!(
        answers("add(a,b,c). not(a). ?- add(a,b,X), not(a)."),
        vec![vec![vec![Value::Symbol("c".into())]]]
    );
}

#[test]
fn negative_queries_and_rules_use_completed_strata() {
    let (p,r)=run("edge(a,b). edge(b,c). node(a). node(b). node(c). node(d). path(X,Y):-edge(X,Y). path(X,Z):-path(X,Y),edge(Y,Z). outside(X):-node(X),not path(a,X). ?- outside(X). ?- not path(c,a).");
    let actual: BTreeSet<_> = r.answers[0].iter().map(|a| a.values[0].clone()).collect();
    assert_eq!(
        actual,
        BTreeSet::from([Value::Symbol("a".into()), Value::Symbol("d".into())])
    );
    assert_eq!(r.answers[1].len(), 1);
    assert!(
        check(&p, &r.certificate(&p))
            .unwrap()
            .condition(Condition::C6)
            .checked
            > 0
    );
}

#[test]
fn aggregate_groups_sets_keys_and_empty_sets_are_explicit() {
    let source="group(a). group(b). row(a,x,5). row(a,y,5). row(a,z,8). total(G,N,S,Distinct):-group(G),count{Item:row(G,Item,V)}=N,sum{V,Item:row(G,Item,V)}=S,sum{V:row(G,Item,V)}=Distinct. ?- total(G,N,S,Distinct).";
    assert_eq!(
        answers(source)[0],
        vec![
            vec![
                Value::Symbol("a".into()),
                Value::Integer(3),
                Value::Integer(18),
                Value::Integer(13)
            ],
            vec![
                Value::Symbol("b".into()),
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(0)
            ],
        ]
    );
    assert!(answers("?- min{V:missing(V)}=N.")[0].is_empty());
    assert!(answers("?- max{V:missing(V)}=N.")[0].is_empty());
}

#[test]
fn aggregate_local_names_do_not_leak_or_capture_later_goals() {
    let (p, r) = run("p(a). p(b). pick(z). ?- count{X:p(X)}=N,pick(X).");
    assert_eq!(p.queries[0].variables, vec!["N", "X"]);
    assert_eq!(
        r.answers[0][0].values,
        vec![Value::Integer(2), Value::Symbol("z".into())]
    );
    assert_eq!(
        answers("p(1). p(2). ?- sum{Y:p(X),@add(X,1,Y)}=N."),
        vec![vec![vec![Value::Integer(5)]]]
    );
    assert_eq!(
        answers("p(a). p(b). blocked(b). ?- count{X:p(X),not blocked(X)}=N."),
        vec![vec![vec![Value::Integer(1)]]]
    );
}

#[test]
fn aggregates_reject_type_errors_and_checked_sum_overflow() {
    for source in [
        "p(a). ?- sum{X:p(X)}=N.",
        "p(9223372036854775807). p(1). ?- sum{X:p(X)}=N.",
    ] {
        assert!(reason(&parse(source).unwrap(), Limits::default()).is_err());
    }
}

#[test]
fn unsafe_modes_non_stratified_cycles_and_value_generating_recursion_are_rejected() {
    for source in [
        "?- @add(X,1,Y).",
        "?- not p(X).",
        "p(X):-not q(X).",
        "p:-not q. q:-p.",
        "p(N):-count{X:p(X)}=N.",
        "p(0). p(N):-p(M),@add(M,1,N).",
        "p(0). p(N):-q(M),@add(M,1,N). q(N):-p(N).",
        "p(0). p(N):-p(M),sum{V:@add(M,1,V)}=N.",
        "?- @unknown(1).",
        "?- @add(1,2).",
        "?- count{X:p(Y)}=N.",
        "?- count{X:count{Y:p(Y)}=X}=N.",
        "?- @fresh(W).",
        "p(W):-@fresh(W),ready.",
        "p(X,W):-p(X,_),@fresh(W).",
        "p(W):-q(W),@fresh(W).",
        "p(~\"forged\").",
    ] {
        assert!(parse(source).is_err(), "accepted {source}");
    }
    run("p(1). p(2). q(X):-p(X),@ge(X,2). p(X):-q(X). ?- q(X).");
}

#[test]
fn witness_identity_is_typed_stable_scoped_and_carried_by_rules() {
    let source="visitor(ada). visitor(bea). ticket(P,W):-visitor(P),@fresh(W). copy(P,W):-ticket(P,W). other(P,W):-visitor(P),@fresh(W). ?- copy(P,W). ?- other(P,W).";
    let (_, r) = run(source);
    let first = &r.answers[0][0].values[1];
    assert!(matches!(first, Value::Witness(_)));
    assert_ne!(first, &r.answers[0][1].values[1]);
    assert_ne!(first, &r.answers[1][0].values[1]);
    assert_eq!(r.answers, run(source).1.answers);
    assert_eq!(answers("p(W):-@fresh(W). ?- p(W).")[0].len(), 1);
}

#[test]
fn full_example_exercises_every_condition_with_no_not_applicable_entries() {
    let (p, r) = run(include_str!("../examples/orders.pl"));
    let report = check(&p, &r.certificate(&p)).unwrap();
    for c in &report.conditions {
        assert_eq!(c.not_applicable, None, "{report}");
        assert!(c.checked > 0, "{report}");
        assert_eq!(c.failed, 0);
    }
    assert_eq!(r.answers[0].len(), 1);
    assert_eq!(r.answers[0][0].values[0], Value::Symbol("ada".into()));
    assert_eq!(r.answers[0][0].values[2], Value::Integer(15));
}

#[test]
fn negative_reachability_and_aggregates_agree_with_an_independent_graph_oracle() {
    for mask in 0..512 {
        let mut reach = [[false; 3]; 3];
        let mut source=String::from("node(n0). node(n1). node(n2). path(X,Y):-edge(X,Y). path(X,Z):-path(X,Y),edge(Y,Z). outside(X):-node(X),not path(n0,X). size(N):-count{X:path(n0,X)}=N. ?- outside(X). ?- size(N).");
        for (i, row) in reach.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                if mask & (1 << (i * 3 + j)) != 0 {
                    *cell = true;
                    source.push_str(&format!("edge(n{i},n{j})."));
                }
            }
        }
        for k in 0..3 {
            for i in 0..3 {
                for j in 0..3 {
                    reach[i][j] |= reach[i][k] && reach[k][j];
                }
            }
        }
        let (_, r) = run(&source);
        let expected: BTreeSet<_> = (0..3)
            .filter(|i| !reach[0][*i])
            .map(|i| Value::Symbol(format!("n{i}")))
            .collect();
        let actual: BTreeSet<_> = r.answers[0].iter().map(|a| a.values[0].clone()).collect();
        assert_eq!(actual, expected, "graph {mask}");
        assert_eq!(
            r.answers[1][0].values,
            vec![Value::Integer(
                reach[0].iter().filter(|v| **v).count() as i64
            )],
            "graph {mask}"
        );
    }
}

#[test]
fn empty_models_and_builtin_only_aggregates_have_valid_closed_boundaries() {
    assert_eq!(answers("?- not missing."), vec![vec![vec![]]]);
    assert_eq!(answers("q:-not missing. ?- q."), vec![vec![vec![]]]);
    assert_eq!(
        answers("total(N):-sum{V:@add(1,2,V)}=N. ?- total(N)."),
        vec![vec![vec![Value::Integer(3)]]]
    );
}

#[test]
fn strata_are_independent_of_source_order_and_merged_files() {
    let rules="eligible(X):-person(X),not blocked(X). blocked(X):-banned(X). total(N):-count{X:eligible(X)}=N. ?- total(N).";
    let facts = "person(a). person(b). banned(b).";
    let expected = answers(&format!("{rules}{facts}"));
    assert_eq!(expected, answers(&format!("{facts}{rules}")));
    let mut p = parse(rules).unwrap();
    p.extend(parse(facts).unwrap());
    let r = reason(&p, Limits::default()).unwrap();
    assert_eq!(r.answers[0][0].values, vec![Value::Integer(1)]);
    assert!(check(&p, &r.certificate(&p)).unwrap().valid());
}

#[test]
fn explicit_checker_limits_cover_evidence_and_closed_model_matching() {
    let (p, r) = run(include_str!("../examples/orders.pl"));
    let c = r.certificate(&p);
    assert!(whye::check_with_limits(
        &p,
        &c,
        Limits {
            max_facts: 1,
            ..Limits::default()
        }
    )
    .is_err());
    assert!(whye::check_with_limits(
        &p,
        &c,
        Limits {
            max_matches: 1,
            ..Limits::default()
        }
    )
    .is_err());
}

#[test]
fn witnesses_flow_through_negation_and_aggregation_across_multiple_boundaries() {
    let (p,r)=run("visitor(a). ticket(P,W):-visitor(P),@fresh(W). valid(W):-ticket(P,W),not revoked(W). total(N):-count{W:valid(W)}=N. ?- total(N).");
    assert_eq!(r.answers[0][0].values, vec![Value::Integer(1)]);
    let c = r.certificate(&p);
    assert_eq!(c.boundaries.len(), 2);
    let report = check(&p, &c).unwrap();
    assert!(report.valid(), "{report}");
    for condition in &report.conditions {
        assert!(condition.checked > 0, "{report}");
        assert_eq!(condition.not_applicable, None);
    }
}
