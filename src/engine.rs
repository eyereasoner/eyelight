use crate::{Atom, Certificate, Error, Goal, Justification, Program, Step, Term, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

pub(crate) type Bindings = BTreeMap<String, Value>;
type Relation = (String, usize);
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
    known: HashSet<Atom>,
    relations: HashMap<Relation, Vec<usize>>,
    positions: HashMap<Position, Vec<usize>>,
}

impl Store {
    fn insert(
        &mut self,
        atom: Atom,
        justification: Justification,
        limit: usize,
    ) -> Result<(), Error> {
        if self.known.contains(&atom) {
            return Ok(());
        }
        if self.steps.len() >= limit {
            return Err(Error::new(format!(
                "Incomplete: fact limit ({limit}) reached; no complete answer is available."
            )));
        }
        let index = self.steps.len();
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
        self.known.insert(atom.clone());
        self.steps.push(Step {
            atom,
            justification,
        });
        Ok(())
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
                Term::Constant(value) => Some(value),
                Term::Variable(name) => bindings.get(name),
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
}

struct Budget {
    used: usize,
    limit: usize,
}
impl Budget {
    fn tick(&mut self) -> Result<(), Error> {
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

struct Match {
    bindings: Bindings,
    premises: Vec<usize>,
}

/// The seed is the newly inserted fact. Every firing is reached when its last
/// missing premise enters the agenda; duplicate conclusions are stored once.
fn join(
    store: &Store,
    goals: &[Goal],
    seed: Option<(usize, usize)>,
    budget: &mut Budget,
) -> Result<Vec<Match>, Error> {
    fn visit(
        store: &Store,
        goals: &[Goal],
        seed: Option<(usize, usize)>,
        at: usize,
        current: Match,
        budget: &mut Budget,
        output: &mut Vec<Match>,
    ) -> Result<(), Error> {
        if at == goals.len() {
            output.push(current);
            return Ok(());
        }
        if let Some((position, fact)) = seed {
            if position == at {
                let mut next = current;
                budget.tick()?;
                if match_goal(&goals[at], &store.steps[fact].atom, &mut next.bindings) {
                    next.premises.push(fact + 1);
                    visit(store, goals, seed, at + 1, next, budget, output)?;
                }
                return Ok(());
            }
        }
        for &fact in store.candidates(&goals[at], &current.bindings) {
            budget.tick()?;
            let mut next = Match {
                bindings: current.bindings.clone(),
                premises: current.premises.clone(),
            };
            if match_goal(&goals[at], &store.steps[fact].atom, &mut next.bindings) {
                next.premises.push(fact + 1);
                visit(store, goals, seed, at + 1, next, budget, output)?;
            }
        }
        Ok(())
    }
    let mut output = Vec::new();
    let mut initial = Bindings::new();
    if let Some((position, fact)) = seed {
        // Bind the seed first so earlier joins can use its argument indexes.
        if !match_goal(&goals[position], &store.steps[fact].atom, &mut initial) {
            return Ok(output);
        }
    }
    visit(
        store,
        goals,
        seed,
        0,
        Match {
            bindings: initial,
            premises: Vec::new(),
        },
        budget,
        &mut output,
    )?;
    Ok(output)
}

pub fn reason(program: &Program, limits: Limits) -> Result<Run, Error> {
    program.validate()?;
    let mut store = Store::default();
    let mut budget = Budget {
        used: 0,
        limit: limits.max_matches,
    };
    for (index, atom) in program.facts.iter().enumerate() {
        store.insert(
            atom.clone(),
            Justification::Source(index + 1),
            limits.max_facts,
        )?;
    }
    let mut triggers: HashMap<Relation, Vec<(usize, usize)>> = HashMap::new();
    for (rule, clause) in program.rules.iter().enumerate() {
        for (position, goal) in clause.body.iter().enumerate() {
            triggers
                .entry((goal.predicate.clone(), goal.terms.len()))
                .or_default()
                .push((rule, position));
        }
    }
    let mut cursor = 0;
    while cursor < store.steps.len() {
        let atom = &store.steps[cursor].atom;
        if let Some(entries) = triggers.get(&(atom.predicate.clone(), atom.values.len())) {
            for &(rule_index, position) in entries {
                let rule = &program.rules[rule_index];
                budget.tick()?;
                for matched in join(&store, &rule.body, Some((position, cursor)), &mut budget)? {
                    let atom = instantiate(&rule.head, &matched.bindings)
                        .ok_or_else(|| Error::new("A rule head remained unbound."))?;
                    store.insert(
                        atom,
                        Justification::Rule {
                            rule: rule_index + 1,
                            premises: matched.premises,
                        },
                        limits.max_facts,
                    )?;
                }
            }
        }
        cursor += 1;
    }
    let mut answers = Vec::new();
    for query in &program.queries {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for matched in join(&store, &query.goals, None, &mut budget)? {
            let values: Vec<_> = query
                .variables
                .iter()
                .map(|name| matched.bindings[name].clone())
                .collect();
            if seen.insert(values.clone()) {
                result.push(Answer {
                    values,
                    premises: matched.premises,
                });
            }
        }
        answers.push(result);
    }
    Ok(Run {
        steps: store.steps,
        answers,
        matches: budget.used,
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
