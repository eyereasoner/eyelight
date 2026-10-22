use crate::language::{plan, Plan, Relation};
use crate::proof::context_goal;
use crate::semantics::{aggregate, bind, builtin, witness, Budget};
use crate::{
    Atom, Boundary, Certificate, Error, Goal, Justification, Literal, Owner, Program, Step, Term,
    Value,
};
use std::collections::{BTreeMap, HashMap, HashSet};

pub(crate) type Bindings = BTreeMap<String, Value>;
type Position = (Relation, usize, Value);
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_facts: usize,
    pub max_matches: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_facts: 1_000_000,
            max_matches: 10_000_000,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub values: Vec<Value>,
    pub premises: Vec<usize>,
}
#[derive(Clone, Debug)]
pub struct Run {
    pub steps: Vec<Step>,
    pub answers: Vec<Vec<Answer>>,
    pub matches: usize,
    pub boundaries: Vec<Boundary>,
}
pub(crate) fn match_goal(goal: &Goal, atom: &Atom, bindings: &mut Bindings) -> bool {
    if goal.predicate != atom.predicate || goal.terms.len() != atom.values.len() {
        return false;
    }
    for (term, value) in goal.terms.iter().zip(&atom.values) {
        match term {
            Term::Constant(expected) if expected != value => return false,
            Term::Variable(name) => {
                if let Some(expected) = bindings.get(name) {
                    if expected != value {
                        return false;
                    }
                } else {
                    bindings.insert(name.clone(), value.clone());
                }
            }
            _ => {}
        }
    }
    true
}

pub(crate) fn instantiate(goal: &Goal, bindings: &Bindings) -> Option<Atom> {
    Some(Atom {
        predicate: goal.predicate.clone(),
        values: goal
            .terms
            .iter()
            .map(|term| match term {
                Term::Constant(value) => Some(value.clone()),
                Term::Variable(name) => bindings.get(name).cloned(),
            })
            .collect::<Option<_>>()?,
    })
}

#[derive(Default)]
struct Store {
    steps: Vec<Step>,
    known: HashMap<Atom, usize>,
    decisions: HashMap<Step, usize>,
    relations: HashMap<Relation, Vec<usize>>,
    positions: HashMap<Position, Vec<usize>>,
    boundaries: Vec<Boundary>,
}
impl Store {
    fn insert(
        &mut self,
        atom: Atom,
        justification: Justification,
        limit: usize,
    ) -> Result<usize, Error> {
        if justification.ordinary() {
            if let Some(id) = self.known.get(&atom) {
                return Ok(*id);
            }
        } else if let Some(id) = self.decisions.get(&Step {
            atom: atom.clone(),
            justification: justification.clone(),
        }) {
            return Ok(*id);
        }
        if self.steps.len() >= limit {
            return Err(Error::new(format!(
                "Incomplete: fact limit ({limit}) reached; no complete answer is available."
            )));
        }
        let index = self.steps.len();
        if justification.ordinary() {
            let relation = (atom.predicate.clone(), atom.values.len());
            self.relations
                .entry(relation.clone())
                .or_default()
                .push(index);
            for (position, value) in atom.values.iter().enumerate() {
                self.positions
                    .entry((relation.clone(), position, value.clone()))
                    .or_default()
                    .push(index);
            }
            self.known.insert(atom.clone(), index + 1);
        } else {
            self.decisions.insert(
                Step {
                    atom: atom.clone(),
                    justification: justification.clone(),
                },
                index + 1,
            );
        }
        self.steps.push(Step {
            atom,
            justification,
        });
        Ok(index + 1)
    }
    fn candidates(&self, goal: &Goal, bindings: &Bindings) -> &[usize] {
        let relation = (goal.predicate.clone(), goal.terms.len());
        let mut result = self
            .relations
            .get(&relation)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for (position, term) in goal.terms.iter().enumerate() {
            let value = match term {
                Term::Constant(v) => Some(v),
                Term::Variable(n) => bindings.get(n),
            };
            if let Some(value) = value {
                let candidates = self
                    .positions
                    .get(&(relation.clone(), position, value.clone()))
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                if candidates.len() < result.len() {
                    result = candidates;
                }
            }
        }
        result
    }
    fn close(&mut self, level: usize, plan: &Plan, budget: &mut Budget) -> Result<(), Error> {
        for _ in &self.steps {
            budget.tick()?;
        }
        let steps = self
            .steps
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.justification.ordinary()
                    && plan.level(&(s.atom.predicate.clone(), s.atom.values.len())) <= level
            })
            .map(|(i, _)| i + 1)
            .collect();
        self.boundaries.push(Boundary { level, steps });
        Ok(())
    }
    fn boundary(&self, level: usize) -> Option<usize> {
        self.boundaries
            .iter()
            .position(|b| b.level == level)
            .map(|i| i + 1)
    }
    fn facts(&self, boundary: usize) -> Vec<Atom> {
        self.boundaries[boundary - 1]
            .steps
            .iter()
            .map(|id| self.steps[id - 1].atom.clone())
            .collect()
    }
    fn materialize(&mut self, evidence: Vec<Evidence>, limit: usize) -> Result<Vec<usize>, Error> {
        evidence
            .into_iter()
            .map(|e| match e {
                Evidence::Fact(id) => Ok(id),
                Evidence::Decision(s) => self.insert(s.atom, s.justification, limit),
            })
            .collect()
    }
}
#[derive(Clone)]
enum Evidence {
    Fact(usize),
    Decision(Step),
}
#[derive(Clone)]
struct Match {
    bindings: Bindings,
    evidence: Vec<Evidence>,
}

struct Join<'a> {
    store: &'a Store,
    program: &'a Program,
    plan: &'a Plan,
    owner: Owner,
    goals: &'a [Literal],
    seed: Option<(usize, usize)>,
}
impl Join<'_> {
    fn visit(
        &self,
        at: usize,
        current: Match,
        budget: &mut Budget,
        out: &mut Vec<Match>,
    ) -> Result<(), Error> {
        if at == self.goals.len() {
            out.push(current);
            return Ok(());
        }
        let literal = &self.goals[at];
        let mut proceed = |bindings, evidence, budget: &mut Budget| {
            let mut next = Match {
                bindings,
                evidence: current.evidence.clone(),
            };
            next.evidence.push(evidence);
            self.visit(at + 1, next, budget, out)
        };
        match literal {
            Literal::Relation(goal) => {
                if let Some((position, fact)) = self.seed {
                    if position == at {
                        budget.tick()?;
                        let mut bindings = current.bindings.clone();
                        if match_goal(goal, &self.store.steps[fact].atom, &mut bindings) {
                            proceed(bindings, Evidence::Fact(fact + 1), budget)?;
                        }
                        return Ok(());
                    }
                }
                for &fact in self.store.candidates(goal, &current.bindings) {
                    budget.tick()?;
                    let mut bindings = current.bindings.clone();
                    if match_goal(goal, &self.store.steps[fact].atom, &mut bindings) {
                        proceed(bindings, Evidence::Fact(fact + 1), budget)?;
                    }
                }
            }
            Literal::Builtin(goal) => {
                budget.tick()?;
                if let Some((atom, bindings)) = builtin(goal, &current.bindings) {
                    proceed(
                        bindings,
                        Evidence::Decision(Step {
                            atom,
                            justification: Justification::Builtin,
                        }),
                        budget,
                    )?;
                }
            }
            Literal::Not(goal) => {
                budget.tick()?;
                let boundary = self
                    .store
                    .boundary(self.plan.boundary(literal))
                    .ok_or_else(|| Error::new("A negative dependency stratum is not complete."))?;
                let atom = instantiate(goal, &current.bindings)
                    .ok_or_else(|| Error::new("Unbound negative goal."))?;
                if !self.store.known.contains_key(&atom) {
                    proceed(
                        current.bindings.clone(),
                        Evidence::Decision(Step {
                            atom,
                            justification: Justification::Absent { boundary },
                        }),
                        budget,
                    )?;
                }
            }
            Literal::Aggregate(a) => {
                let boundary = self
                    .store
                    .boundary(self.plan.boundary(literal))
                    .ok_or_else(|| {
                        Error::new("An aggregate dependency stratum is not complete.")
                    })?;
                if let Some(result) =
                    aggregate(a, &current.bindings, &self.store.facts(boundary), budget)?
                {
                    let mut bindings = current.bindings.clone();
                    if bind(&a.result, result, &mut bindings) {
                        let goal = context_goal(self.program, self.owner, at + 1)
                            .ok_or_else(|| Error::new("Invalid aggregate source context."))?;
                        let atom = instantiate(&goal, &bindings)
                            .ok_or_else(|| Error::new("Unbound aggregate grouping variable."))?;
                        proceed(
                            bindings,
                            Evidence::Decision(Step {
                                atom,
                                justification: Justification::Aggregate {
                                    owner: self.owner,
                                    position: at + 1,
                                    boundary,
                                },
                            }),
                            budget,
                        )?;
                    }
                }
            }
            Literal::Fresh(term) => {
                budget.tick()?;
                let Owner::Rule(rule) = self.owner else {
                    return Err(Error::new("@fresh needs a rule context."));
                };
                let mut bindings = current.bindings.clone();
                if bind(
                    term,
                    witness(rule, at + 1, &current.bindings),
                    &mut bindings,
                ) {
                    let goal = context_goal(self.program, self.owner, at + 1)
                        .ok_or_else(|| Error::new("Invalid witness source context."))?;
                    let atom = instantiate(&goal, &bindings)
                        .ok_or_else(|| Error::new("Unbound witness firing variable."))?;
                    proceed(
                        bindings,
                        Evidence::Decision(Step {
                            atom,
                            justification: Justification::Fresh {
                                rule,
                                position: at + 1,
                            },
                        }),
                        budget,
                    )?;
                }
            }
        }
        Ok(())
    }
    fn run(&self, budget: &mut Budget) -> Result<Vec<Match>, Error> {
        let mut initial = Bindings::new();
        // Seed prebinding is safe for a positive conjunction. Other bodies keep
        // source order, preserving aggregate-local scopes and witness context.
        if self.goals.iter().all(|g| matches!(g, Literal::Relation(_))) {
            if let Some((position, fact)) = self.seed {
                if let Literal::Relation(g) = &self.goals[position] {
                    if !match_goal(g, &self.store.steps[fact].atom, &mut initial) {
                        return Ok(Vec::new());
                    }
                }
            }
        }
        let mut out = Vec::new();
        self.visit(
            0,
            Match {
                bindings: initial,
                evidence: Vec::new(),
            },
            budget,
            &mut out,
        )?;
        Ok(out)
    }
}

fn fire(
    store: &mut Store,
    program: &Program,
    rule: usize,
    matched: Match,
    limit: usize,
) -> Result<(), Error> {
    let atom = instantiate(&program.rules[rule].head, &matched.bindings)
        .ok_or_else(|| Error::new("A rule head remained unbound."))?;
    if !store.known.contains_key(&atom) {
        let premises = store.materialize(matched.evidence, limit)?;
        store.insert(
            atom,
            Justification::Rule {
                rule: rule + 1,
                premises,
            },
            limit,
        )?;
    }
    Ok(())
}

pub fn reason(program: &Program, limits: Limits) -> Result<Run, Error> {
    let plan = plan(program)?;
    let mut store = Store::default();
    let mut budget = Budget {
        used: 0,
        limit: limits.max_matches,
    };
    for (i, atom) in program.facts.iter().enumerate() {
        store.insert(atom.clone(), Justification::Source(i + 1), limits.max_facts)?;
    }
    let max = plan.levels.values().copied().max().unwrap_or(0);
    for level in 0..=max {
        let mut triggers: HashMap<Relation, Vec<(usize, usize)>> = HashMap::new();
        for (rule, clause) in program.rules.iter().enumerate() {
            if plan.rules[rule] != level {
                continue;
            }
            let mut positive = false;
            for (position, g) in clause.body.iter().enumerate() {
                if let Literal::Relation(g) = g {
                    positive = true;
                    triggers
                        .entry((g.predicate.clone(), g.terms.len()))
                        .or_default()
                        .push((rule, position));
                }
            }
            if !positive {
                let matches = Join {
                    store: &store,
                    program,
                    plan: &plan,
                    owner: Owner::Rule(rule + 1),
                    goals: &clause.body,
                    seed: None,
                }
                .run(&mut budget)?;
                for matched in matches {
                    fire(&mut store, program, rule, matched, limits.max_facts)?;
                }
            }
        }
        let mut cursor = 0;
        while cursor < store.steps.len() {
            let step = &store.steps[cursor];
            if step.justification.ordinary() {
                if let Some(entries) =
                    triggers.get(&(step.atom.predicate.clone(), step.atom.values.len()))
                {
                    for &(rule, position) in entries {
                        budget.tick()?;
                        let matches = Join {
                            store: &store,
                            program,
                            plan: &plan,
                            owner: Owner::Rule(rule + 1),
                            goals: &program.rules[rule].body,
                            seed: Some((position, cursor)),
                        }
                        .run(&mut budget)?;
                        for matched in matches {
                            fire(&mut store, program, rule, matched, limits.max_facts)?;
                        }
                    }
                }
            }
            cursor += 1;
        }
        let needed = program
            .rules
            .iter()
            .flat_map(|r| &r.body)
            .chain(program.queries.iter().flat_map(|q| &q.goals))
            .any(|g| {
                matches!(g, Literal::Not(_) | Literal::Aggregate(_)) && plan.boundary(g) == level
            });
        if needed {
            store.close(level, &plan, &mut budget)?;
        }
    }
    let mut answers = Vec::new();
    for (index, query) in program.queries.iter().enumerate() {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for matched in (Join {
            store: &store,
            program,
            plan: &plan,
            owner: Owner::Query(index + 1),
            goals: &query.goals,
            seed: None,
        })
        .run(&mut budget)?
        {
            let values: Vec<_> = query
                .variables
                .iter()
                .map(|n| matched.bindings[n].clone())
                .collect();
            if seen.insert(values.clone()) {
                let premises = store.materialize(matched.evidence, limits.max_facts)?;
                result.push(Answer { values, premises });
            }
        }
        answers.push(result);
    }
    Ok(Run {
        steps: store.steps,
        answers,
        matches: budget.used,
        boundaries: store.boundaries,
    })
}
pub(crate) fn query_text(query: &crate::Query) -> String {
    format!(
        "?- {}.",
        query
            .goals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

impl Run {
    pub fn output(&self, program: &Program) -> String {
        let mut text = String::new();
        if program.queries.is_empty() {
            for step in &self.steps {
                if matches!(step.justification, Justification::Rule { .. }) {
                    text.push_str(&format!("{}.\n", step.atom));
                }
            }
        } else {
            for (index, query) in program.queries.iter().enumerate() {
                if index > 0 {
                    text.push('\n');
                }
                text.push_str(&format!("{}\n", query_text(query)));
                if self.answers[index].is_empty() {
                    text.push_str("false.\n");
                }
                for answer in &self.answers[index] {
                    if query.variables.is_empty() {
                        text.push_str("true.\n");
                    } else {
                        text.push_str(
                            &query
                                .variables
                                .iter()
                                .zip(&answer.values)
                                .map(|(name, value)| format!("{name} = {value}"))
                                .collect::<Vec<_>>()
                                .join(", "),
                        );
                        text.push_str(".\n");
                    }
                }
            }
        }
        text
    }
    pub fn explain(&self, program: &Program) -> String {
        let certificate = self.certificate(program);
        let mut text = self.output(program);
        text.push_str("\nExplanation\n");
        let rules: std::collections::BTreeSet<_> = certificate
            .steps
            .iter()
            .filter_map(|step| match step.justification {
                Justification::Rule { rule, .. } => Some(rule),
                _ => None,
            })
            .collect();
        for rule in rules {
            text.push_str(&format!("Rule {rule}: {}\n", program.rules[rule - 1]));
        }
        for (index, step) in certificate.steps.iter().enumerate() {
            let justification = match &step.justification {
                Justification::Source(source) => format!("source fact {source}"),
                Justification::Rule { rule, premises } => format!(
                    "rule {rule}; uses {}",
                    premises
                        .iter()
                        .map(|id| format!("[{id}]"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Justification::Builtin => "built-in re-decision".into(),
                Justification::Absent { boundary } => format!("absent from boundary {boundary}"),
                Justification::Aggregate { boundary, .. } => {
                    format!("aggregate over boundary {boundary}")
                }
                Justification::Fresh { rule, .. } => format!("fresh witness for rule {rule}"),
            };
            text.push_str(&format!(
                "[{}] {}. ({justification})\n",
                index + 1,
                step.atom
            ));
        }
        if certificate.claims.is_empty() {
            text.push_str("No positive claims. This is not a proof of absence.\n");
        }
        text
    }
    pub fn certificate(&self, program: &Program) -> Certificate {
        Certificate::from_run(program, self)
    }
}
