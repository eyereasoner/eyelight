use eyelight::{check, parse, reason, Certificate, Claim, Justification, Limits};

const PROGRAM: &str = "p(a). p(b). edge(a,b). q(X) :- p(X). r(X,Y) :- q(X), edge(X,Y). ?- r(X,Y).";

fn certificate() -> Certificate {
    let p = parse(PROGRAM).unwrap();
    reason(&p, Limits::default()).unwrap().certificate(&p)
}
fn reject(certificate: &Certificate) {
    assert!(
        !check(&parse(PROGRAM).unwrap(), certificate)
            .unwrap()
            .valid(),
        "accepted {certificate}"
    );
}

#[test]
fn forged_source_fact_is_rejected() {
    let mut c = certificate();
    c.steps[0].atom.values[0] = eyelight::Value::Symbol("forged".into());
    reject(&c);
}
#[test]
fn unknown_and_zero_source_ids_are_rejected() {
    for id in [0, 999] {
        let mut c = certificate();
        c.steps[0].justification = Justification::Source(id);
        reject(&c);
    }
}
#[test]
fn unknown_and_zero_rules_are_rejected() {
    for id in [0, 999] {
        let mut c = certificate();
        c.steps.last_mut().unwrap().justification = Justification::Rule {
            rule: id,
            premises: vec![1],
        };
        reject(&c);
    }
}
#[test]
fn cycles_and_forward_dependencies_are_rejected() {
    let mut c = certificate();
    let last = c.steps.len();
    c.steps.last_mut().unwrap().justification = Justification::Rule {
        rule: 2,
        premises: vec![last, 1],
    };
    reject(&c);
}
#[test]
fn missing_rule_premises_are_rejected() {
    let mut c = certificate();
    if let Justification::Rule { premises, .. } = &mut c.steps.last_mut().unwrap().justification {
        premises.pop();
    }
    reject(&c);
}
#[test]
fn wrong_head_and_inconsistent_rule_bindings_are_rejected() {
    let p = parse("p(a). edge(b,c). r(X,Y) :- p(X), edge(X,Y). ?- r(X,Y).").unwrap();
    let forged = Certificate::parse("eyelight-proof 1\nstep 1 source 1 : p(a).\nstep 2 source 2 : edge(b,c).\nstep 3 rule 1 [1,2] : r(a,c).\nanswer 1 [3]\n").unwrap();
    assert!(!check(&p, &forged).unwrap().valid());
    let mut c = certificate();
    c.steps.last_mut().unwrap().atom.values.reverse();
    reject(&c);
}
#[test]
fn incomplete_query_conjunction_is_rejected() {
    let p = parse("p(a). q(b). ?- p(X), q(X).").unwrap();
    for claim in ["answer 1 [1]", "answer 1 [1,2]"] {
        let c = Certificate::parse(&format!(
            "eyelight-proof 1\nstep 1 source 1 : p(a).\nstep 2 source 2 : q(b).\n{claim}\n"
        ))
        .unwrap();
        assert!(!check(&p, &c).unwrap().valid());
    }
}
#[test]
fn unknown_claims_and_duplicate_answers_are_rejected() {
    let mut c = certificate();
    c.claims.push(c.claims[0].clone());
    reject(&c);
    for query in [0, 999] {
        let mut c = certificate();
        c.claims[0] = Claim::Answer {
            query,
            premises: vec![1],
        };
        reject(&c);
    }
    let mut c = certificate();
    c.claims[0] = Claim::Derived(1);
    reject(&c);
}
#[test]
fn irrelevant_steps_and_duplicate_steps_are_rejected() {
    let mut c = certificate();
    c.claims.clear();
    reject(&c);
    let mut c = certificate();
    c.steps.push(c.steps[0].clone());
    reject(&c);
}
#[test]
fn missing_and_zero_query_evidence_is_rejected() {
    for id in [0, 999] {
        let mut c = certificate();
        c.claims[0] = Claim::Answer {
            query: 1,
            premises: vec![id],
        };
        reject(&c);
    }
}
#[test]
fn malformed_metadata_and_variable_conclusions_are_rejected() {
    for text in [
        "eyelight-proof 2",
        "eyelight-proof 1\nstep 0 source 1 : p(a).",
        "eyelight-proof 1\nstep 1 trusted 1 : p(a).",
        "eyelight-proof 1\nstep 1 source 1 : p(X).",
        "eyelight-proof 1\nstep 1 source 0 : p(a).",
        "eyelight-proof 1\nstep 1 source 1 extra : p(a).",
        "eyelight-proof 1\nanswer 1 [0]",
        "eyelight-proof 1\nunknown",
    ] {
        assert!(Certificate::parse(text).is_err(), "accepted {text}");
    }
}
#[test]
fn valid_does_not_mean_complete_or_prove_absence() {
    let p = parse(PROGRAM).unwrap();
    let empty = Certificate {
        boundaries: vec![],
        steps: vec![],
        claims: vec![],
    };
    assert_eq!(check(&p, &empty).unwrap().claims, 0);
    assert!(check(&p, &certificate())
        .unwrap()
        .to_string()
        .contains("completeness is not checked"));
}
