use crate::model::term_variables;
use crate::{Error, Goal, Literal, Program, Term, Value};
use std::collections::{BTreeSet, HashMap, HashSet};

pub(crate) type Relation = (String, usize);
pub(crate) fn relation(goal: &Goal) -> Relation {
    (goal.predicate.clone(), goal.terms.len())
}

pub(crate) struct Plan {
    pub levels: HashMap<Relation, usize>,
    pub rules: Vec<usize>,
}
impl Plan {
    pub fn level(&self, relation: &Relation) -> usize {
        self.levels.get(relation).copied().unwrap_or(0)
    }
    pub fn boundary(&self, literal: &Literal) -> usize {
        let mut dependencies = Vec::new();
        dependencies_of(literal, false, &mut dependencies);
        dependencies
            .iter()
            .map(|(r, _)| self.level(r))
            .max()
            .unwrap_or(0)
    }
}

/// Built-ins have explicit input modes; arithmetic relations never enumerate an
/// unbounded domain. The optional final argument is a deterministic output.
pub(crate) fn builtin_inputs(goal: &Goal) -> Result<usize, Error> {
    let (arity, inputs) = match goal.predicate.as_str() {
        "add" | "sub" | "mul" | "div" | "mod" | "concat" => (3, 2),
        "abs" | "strlen" => (2, 1),
        "eq" | "neq" | "lt" | "le" | "gt" | "ge" => (2, 2),
        name => return Err(Error::new(format!("Unknown built-in @{name}."))),
    };
    if goal.terms.len() != arity {
        return Err(Error::new(format!(
            "@{} needs {arity} arguments.",
            goal.predicate
        )));
    }
    Ok(inputs)
}

fn source_term(term: &Term) -> Result<(), Error> {
    if matches!(term, Term::Constant(Value::Witness(_))) {
        return Err(Error::new(
            "Witness constants belong to certificates, not source programs.",
        ));
    }
    Ok(())
}
fn goal_shape(goal: &Goal) -> Result<(), Error> {
    if goal.terms.len() > 64 {
        return Err(Error::new("A relation supports at most 64 arguments."));
    }
    for t in &goal.terms {
        source_term(t)?;
    }
    Ok(())
}

/// Left-to-right safety also defines aggregate grouping: an already bound name
/// is a parameter; new names inside braces have only that aggregate's scope.
pub(crate) fn bound_after(
    body: &[Literal],
    mut bound: BTreeSet<String>,
    fresh: bool,
    nested: bool,
) -> Result<(BTreeSet<String>, bool), Error> {
    if body.is_empty() || body.len() > 64 {
        return Err(Error::new("Use 1 to 64 body goals."));
    }
    let mut generates = false;
    for (position, literal) in body.iter().enumerate() {
        match literal {
            Literal::Relation(g) => {
                goal_shape(g)?;
                bound.extend(g.variables());
            }
            Literal::Builtin(g) => {
                goal_shape(g)?;
                let inputs = builtin_inputs(g)?;
                if !term_variables(&g.terms[..inputs]).is_subset(&bound) {
                    return Err(Error::new(format!(
                        "@{} inputs must be bound by earlier goals.",
                        g.predicate
                    )));
                }
                if !term_variables(&g.terms[inputs..]).is_subset(&bound) {
                    generates = true;
                }
                bound.extend(g.variables());
            }
            Literal::Not(g) => {
                goal_shape(g)?;
                if !g.variables().is_subset(&bound) {
                    return Err(Error::new(
                        "Every variable in a negated goal must be bound by an earlier goal.",
                    ));
                }
            }
            Literal::Aggregate(a) => {
                if nested {
                    return Err(Error::new(
                        "Nested aggregates are not supported; use an intermediate relation.",
                    ));
                }
                if a.template.is_empty() || a.template.len() > 64 {
                    return Err(Error::new("An aggregate needs 1 to 64 template terms."));
                }
                for t in a.template.iter().chain(std::iter::once(&a.result)) {
                    source_term(t)?;
                }
                let (inner, _) = bound_after(&a.body, bound.clone(), false, true)?;
                if !term_variables(&a.template).is_subset(&inner) {
                    return Err(Error::new(
                        "Every aggregate template variable must be bound by its goals.",
                    ));
                }
                bound.extend(term_variables(std::slice::from_ref(&a.result)));
                generates = true;
            }
            Literal::Fresh(t) => {
                if !fresh || nested || position + 1 != body.len() {
                    return Err(Error::new(
                        "@fresh is allowed only as the final goal of a rule.",
                    ));
                }
                let Term::Variable(name) = t else {
                    return Err(Error::new("@fresh needs a new output variable."));
                };
                if bound.contains(name) {
                    return Err(Error::new(
                        "@fresh needs a variable not bound by an earlier goal.",
                    ));
                }
                bound.insert(name.clone());
                generates = true;
            }
        }
    }
    Ok((bound, generates))
}

fn dependencies_of(literal: &Literal, strict: bool, output: &mut Vec<(Relation, bool)>) {
    match literal {
        Literal::Relation(g) => output.push((relation(g), strict)),
        Literal::Not(g) => output.push((relation(g), true)),
        Literal::Aggregate(a) => {
            for g in &a.body {
                dependencies_of(g, true, output);
            }
        }
        _ => {}
    }
}

pub(crate) fn plan(program: &Program) -> Result<Plan, Error> {
    let mut levels = HashMap::new();
    let mut edges = Vec::new();
    let mut generating = Vec::new();
    for atom in &program.facts {
        if atom.values.len() > 64 {
            return Err(Error::new("A relation supports at most 64 arguments."));
        }
        if atom.values.iter().any(|v| matches!(v, Value::Witness(_))) {
            return Err(Error::new("Source facts cannot supply witness identities."));
        }
        levels.insert((atom.predicate.clone(), atom.values.len()), 0);
    }
    for (i, rule) in program.rules.iter().enumerate() {
        goal_shape(&rule.head)?;
        let (bound, generates) = bound_after(&rule.body, BTreeSet::new(), true, false)?;
        if !rule.head.variables().is_subset(&bound) {
            return Err(Error::new(format!(
                "Rule {} has a head variable absent from its bound body.",
                i + 1
            )));
        }
        let head = relation(&rule.head);
        let minimum = usize::from(rule.body.iter().any(|g| matches!(g, Literal::Aggregate(_))));
        let level = levels.entry(head.clone()).or_insert(0);
        *level = (*level).max(minimum);
        let mut deps = Vec::new();
        for literal in &rule.body {
            dependencies_of(literal, false, &mut deps);
        }
        for (dependency, strict) in deps {
            levels.entry(dependency.clone()).or_insert(0);
            edges.push((head.clone(), dependency, strict));
        }
        if generates {
            generating.push(head);
        }
    }
    for (i, query) in program.queries.iter().enumerate() {
        let (bound, _) = bound_after(&query.goals, BTreeSet::new(), false, false)?;
        let projected: BTreeSet<_> = query.variables.iter().cloned().collect();
        if projected.len() != query.variables.len() || !projected.is_subset(&bound) {
            return Err(Error::new(format!(
                "Query {} has invalid projected variables.",
                i + 1
            )));
        }
        for literal in &query.goals {
            let mut deps = Vec::new();
            dependencies_of(literal, false, &mut deps);
            for (dependency, _) in deps {
                levels.entry(dependency).or_insert(0);
            }
        }
    }
    // A strict dependency cycle keeps raising a level. Positive cycles settle
    // at one level and remain ordinary finite Datalog recursion.
    let count = levels.len();
    for pass in 0..=count {
        let mut changed = false;
        for (head, dependency, strict) in &edges {
            let next = levels[dependency] + usize::from(*strict);
            if levels[head] < next {
                levels.insert(head.clone(), next);
                changed = true;
            }
        }
        if !changed {
            break;
        }
        if pass == count {
            return Err(Error::new("Negation and aggregation must be stratified: a dependency cycle crosses a negative or aggregate goal."));
        }
    }
    for head in generating {
        let mut pending: Vec<_> = edges
            .iter()
            .filter(|(h, _, _)| h == &head)
            .map(|(_, d, _)| d.clone())
            .collect();
        let mut seen = HashSet::new();
        while let Some(current) = pending.pop() {
            if current == head {
                return Err(Error::new("Value-producing built-ins and @fresh cannot define recursive relations; use a separate nonrecursive rule."));
            }
            if seen.insert(current.clone()) {
                pending.extend(
                    edges
                        .iter()
                        .filter(|(h, _, _)| h == &current)
                        .map(|(_, d, _)| d.clone()),
                );
            }
        }
    }
    let rules = program
        .rules
        .iter()
        .map(|r| levels[&relation(&r.head)])
        .collect();
    Ok(Plan { levels, rules })
}
