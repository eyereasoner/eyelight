use crate::engine::{instantiate, match_goal, Bindings};
use crate::{Aggregate, AggregateKind, Atom, Error, Goal, Literal, Term, Value};
use std::collections::BTreeSet;

pub(crate) struct Budget {
    pub used: usize,
    pub limit: usize,
}
impl Budget {
    pub fn tick(&mut self) -> Result<(), Error> {
        if self.used >= self.limit {
            return Err(Error::new(format!(
                "Incomplete: match limit ({}) reached; no complete answer is available.",
                self.limit
            )));
        }
        self.used += 1;
        Ok(())
    }
}

pub(crate) fn value(term: &Term, bindings: &Bindings) -> Option<Value> {
    match term {
        Term::Constant(v) => Some(v.clone()),
        Term::Variable(n) => bindings.get(n).cloned(),
    }
}
pub(crate) fn bind(term: &Term, value: Value, bindings: &mut Bindings) -> bool {
    match term {
        Term::Constant(v) => v == &value,
        Term::Variable(n) => {
            if let Some(v) = bindings.get(n) {
                v == &value
            } else {
                bindings.insert(n.clone(), value);
                true
            }
        }
    }
}

pub(crate) fn builtin(goal: &Goal, bindings: &Bindings) -> Option<(Atom, Bindings)> {
    let inputs = crate::language::builtin_inputs(goal).ok()?;
    let args: Vec<_> = goal.terms[..inputs]
        .iter()
        .map(|t| value(t, bindings))
        .collect::<Option<_>>()?;
    let output = match (goal.predicate.as_str(), args.as_slice()) {
        ("add", [Value::Integer(a), Value::Integer(b)]) => Some(Value::Integer(a.checked_add(*b)?)),
        ("sub", [Value::Integer(a), Value::Integer(b)]) => Some(Value::Integer(a.checked_sub(*b)?)),
        ("mul", [Value::Integer(a), Value::Integer(b)]) => Some(Value::Integer(a.checked_mul(*b)?)),
        ("div", [Value::Integer(a), Value::Integer(b)]) => Some(Value::Integer(a.checked_div(*b)?)),
        ("mod", [Value::Integer(a), Value::Integer(b)]) => Some(Value::Integer(a.checked_rem(*b)?)),
        ("abs", [Value::Integer(a)]) => Some(Value::Integer(a.checked_abs()?)),
        ("concat", [Value::Text(a), Value::Text(b)]) => Some(Value::Text(format!("{a}{b}"))),
        ("strlen", [Value::Text(a)]) => {
            Some(Value::Integer(i64::try_from(a.chars().count()).ok()?))
        }
        ("eq", [a, b]) if a == b => None,
        ("neq", [a, b]) if a != b => None,
        ("lt", [Value::Integer(a), Value::Integer(b)]) if a < b => None,
        ("le", [Value::Integer(a), Value::Integer(b)]) if a <= b => None,
        ("gt", [Value::Integer(a), Value::Integer(b)]) if a > b => None,
        ("ge", [Value::Integer(a), Value::Integer(b)]) if a >= b => None,
        _ => return None,
    };
    let mut next = bindings.clone();
    if let Some(output) = output {
        if !bind(&goal.terms[inputs], output, &mut next) {
            return None;
        }
    }
    Some((instantiate(goal, &next)?, next))
}

/// Finite matching over an already supplied model, without adding facts or
/// searching for their derivations. Used for closure and aggregate re-decision.
pub(crate) fn model_solutions(
    body: &[Literal],
    initial: Bindings,
    facts: &[Atom],
    budget: &mut Budget,
) -> Result<Vec<Bindings>, Error> {
    fn visit(
        body: &[Literal],
        at: usize,
        bindings: Bindings,
        facts: &[Atom],
        budget: &mut Budget,
        out: &mut Vec<Bindings>,
    ) -> Result<(), Error> {
        if at == body.len() {
            out.push(bindings);
            return Ok(());
        }
        match &body[at] {
            Literal::Relation(goal) => {
                for atom in facts {
                    if goal.predicate != atom.predicate || goal.terms.len() != atom.values.len() {
                        continue;
                    }
                    budget.tick()?;
                    let mut next = bindings.clone();
                    if match_goal(goal, atom, &mut next) {
                        visit(body, at + 1, next, facts, budget, out)?;
                    }
                }
            }
            Literal::Builtin(goal) => {
                budget.tick()?;
                if let Some((_, next)) = builtin(goal, &bindings) {
                    visit(body, at + 1, next, facts, budget, out)?;
                }
            }
            Literal::Not(goal) => {
                let atom = instantiate(goal, &bindings)
                    .ok_or_else(|| Error::new("Unbound negative goal."))?;
                let mut present = false;
                for fact in facts {
                    budget.tick()?;
                    if fact == &atom {
                        present = true;
                        break;
                    }
                }
                if !present {
                    visit(body, at + 1, bindings, facts, budget, out)?;
                }
            }
            Literal::Aggregate(a) => {
                if let Some(result) = aggregate(a, &bindings, facts, budget)? {
                    let mut next = bindings;
                    if bind(&a.result, result, &mut next) {
                        visit(body, at + 1, next, facts, budget, out)?;
                    }
                }
            }
            Literal::Fresh(_) => {
                return Err(Error::new("A fresh witness needs its source rule context."))
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    visit(body, 0, initial, facts, budget, &mut out)?;
    Ok(out)
}

pub(crate) fn aggregate(
    a: &Aggregate,
    outer: &Bindings,
    facts: &[Atom],
    budget: &mut Budget,
) -> Result<Option<Value>, Error> {
    let mut tuples = BTreeSet::new();
    for bindings in model_solutions(&a.body, outer.clone(), facts, budget)? {
        let tuple = a
            .template
            .iter()
            .map(|t| value(t, &bindings))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| Error::new("Unbound aggregate template."))?;
        tuples.insert(tuple);
    }
    if a.kind == AggregateKind::Count {
        return Ok(i64::try_from(tuples.len()).ok().map(Value::Integer));
    }
    let mut numbers = Vec::new();
    for tuple in tuples {
        let Value::Integer(n) = tuple[0] else {
            return Err(Error::new(format!(
                "{} requires integer values in the first template position.",
                a.kind
            )));
        };
        numbers.push(n);
    }
    let result = match a.kind {
        AggregateKind::Sum => Some(
            numbers
                .into_iter()
                .try_fold(0i64, i64::checked_add)
                .ok_or_else(|| {
                    Error::new("Aggregate sum exceeds the signed 64-bit integer range.")
                })?,
        ),
        AggregateKind::Min => numbers.into_iter().min(),
        AggregateKind::Max => numbers.into_iter().max(),
        AggregateKind::Count => unreachable!(),
    };
    Ok(result.map(Value::Integer))
}

pub(crate) fn witness(rule: usize, position: usize, bindings: &Bindings) -> Value {
    // Debug's tagged, escaped value representation keeps kinds and delimiters
    // distinct; this is a scoped identity, not a hash that could collide.
    Value::Witness(format!("r{rule}:g{position}:{bindings:?}"))
}
