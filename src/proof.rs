use crate::engine::{instantiate, match_goal, Bindings};
use crate::{parse_atom, Atom, Error, Goal, Program, Run, Value};
use std::collections::{BTreeSet, HashSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Justification {
    Source(usize),
    Rule { rule: usize, premises: Vec<usize> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub atom: Atom,
    pub justification: Justification,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Claim {
    Answer { query: usize, premises: Vec<usize> },
    Derived(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Certificate {
    pub steps: Vec<Step>,
    pub claims: Vec<Claim>,
}

impl Certificate {
    pub(crate) fn from_run(program: &Program, run: &Run) -> Self {
        let mut claims: Vec<_> = if program.queries.is_empty() {
            run.steps
                .iter()
                .enumerate()
                .filter_map(|(index, step)| match step.justification {
                    Justification::Rule { .. } => Some(Claim::Derived(index + 1)),
                    _ => None,
                })
                .collect()
        } else {
            run.answers
                .iter()
                .enumerate()
                .flat_map(|(query, answers)| {
                    answers.iter().map(move |answer| Claim::Answer {
                        query: query + 1,
                        premises: answer.premises.clone(),
                    })
                })
                .collect()
        };
        let mut needed = BTreeSet::new();
        let mut pending: Vec<_> = claims.iter().flat_map(claim_premises).collect();
        while let Some(id) = pending.pop() {
            if needed.insert(id) {
                if let Justification::Rule { premises, .. } = &run.steps[id - 1].justification {
                    pending.extend(premises);
                }
            }
        }
        let mut renaming = vec![0; run.steps.len() + 1];
        for (index, id) in needed.iter().enumerate() {
            renaming[*id] = index + 1;
        }
        let steps = needed
            .into_iter()
            .map(|id| {
                let mut step = run.steps[id - 1].clone();
                if let Justification::Rule { premises, .. } = &mut step.justification {
                    for premise in premises {
                        *premise = renaming[*premise];
                    }
                }
                step
            })
            .collect();
        for claim in &mut claims {
            match claim {
                Claim::Derived(id) => *id = renaming[*id],
                Claim::Answer { premises, .. } => {
                    for premise in premises {
                        *premise = renaming[*premise];
                    }
                }
            }
        }
        Self { steps, claims }
    }

    pub fn parse(source: &str) -> Result<Self, Error> {
        Self::read(source)
            .map_err(|error| Error::new(format!("Invalid proof [C3 justification]: {error}")))
    }

    fn read(source: &str) -> Result<Self, Error> {
        let mut lines = source.lines();
        if lines.next() != Some("eyelight-proof 1") {
            return Err(Error::new("Expected certificate header: eyelight-proof 1."));
        }
        let mut certificate = Self {
            steps: Vec::new(),
            claims: Vec::new(),
        };
        let mut in_claims = false;
        for (line_number, line) in lines.enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let invalid = || {
                Error::new(format!(
                    "Certificate line {}: invalid record.",
                    line_number + 2
                ))
            };
            if line.starts_with("step ") {
                if in_claims {
                    return Err(invalid());
                }
                let (metadata, atom) = line.split_once(" : ").ok_or_else(invalid)?;
                let fields: Vec<_> = metadata.split_whitespace().collect();
                if fields.len() < 4 || number(fields[1])? != certificate.steps.len() + 1 {
                    return Err(invalid());
                }
                let justification = match fields[2] {
                    "source" if fields.len() == 4 => Justification::Source(number(fields[3])?),
                    "rule" if fields.len() == 5 => Justification::Rule {
                        rule: number(fields[3])?,
                        premises: references(fields[4])?,
                    },
                    _ => return Err(invalid()),
                };
                certificate.steps.push(Step {
                    atom: parse_atom(atom)?,
                    justification,
                });
            } else {
                in_claims = true;
                let fields: Vec<_> = line.split_whitespace().collect();
                let claim = match fields.as_slice() {
                    ["answer", query, premises] => Claim::Answer {
                        query: number(query)?,
                        premises: references(premises)?,
                    },
                    ["derived", id] => Claim::Derived(number(id)?),
                    _ => return Err(invalid()),
                };
                certificate.claims.push(claim);
            }
        }
        Ok(certificate)
    }
}

fn number(text: &str) -> Result<usize, Error> {
    text.parse::<usize>()
        .ok()
        .filter(|number| *number > 0)
        .ok_or_else(|| Error::new("Certificate IDs must be positive integers."))
}
fn references(text: &str) -> Result<Vec<usize>, Error> {
    let inner = text
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
        .ok_or_else(|| Error::new("Put premise IDs inside [brackets], separated by commas."))?;
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner.split(',').map(number).collect()
}
fn list(f: &mut fmt::Formatter<'_>, ids: &[usize]) -> fmt::Result {
    f.write_str("[")?;
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            f.write_str(",")?;
        }
        write!(f, "{id}")?;
    }
    f.write_str("]")
}
impl fmt::Display for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "eyelight-proof 1")?;
        for (index, step) in self.steps.iter().enumerate() {
            write!(f, "step {} ", index + 1)?;
            match &step.justification {
                Justification::Source(source) => write!(f, "source {source}")?,
                Justification::Rule { rule, premises } => {
                    write!(f, "rule {rule} ")?;
                    list(f, premises)?;
                }
            }
            writeln!(f, " : {}.", step.atom)?;
        }
        for claim in &self.claims {
            match claim {
                Claim::Answer { query, premises } => {
                    write!(f, "answer {query} ")?;
                    list(f, premises)?;
                    writeln!(f)?;
                }
                Claim::Derived(id) => writeln!(f, "derived {id}")?,
            }
        }
        Ok(())
    }
}

fn claim_premises(claim: &Claim) -> Vec<usize> {
    match claim {
        Claim::Derived(id) => vec![*id],
        Claim::Answer { premises, .. } => premises.clone(),
    }
}

/// The nine proof conditions, specialized to eyelight's language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
}

impl Condition {
    pub const ALL: [Self; 9] = [
        Self::C1,
        Self::C2,
        Self::C3,
        Self::C4,
        Self::C5,
        Self::C6,
        Self::C7,
        Self::C8,
        Self::C9,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::C1 => "resolution",
            Self::C2 => "well-foundedness",
            Self::C3 => "justification",
            Self::C4 => "coverage",
            Self::C5 => "re-decision",
            Self::C6 => "boundary consistency",
            Self::C7 => "relevance",
            Self::C8 => "witness integrity",
            Self::C9 => "query isolation",
        }
    }
}
impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{} {}", *self as usize + 1, self.name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofLocation {
    Step(usize),
    Claim(usize),
}
impl fmt::Display for ProofLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Step(id) => write!(f, "step {id}"),
            Self::Claim(id) => write!(f, "claim {id}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub condition: Condition,
    pub location: ProofLocation,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionReport {
    pub condition: Condition,
    /// Attempted checks, including those that failed. Dependent checks with
    /// missing evidence are skipped; they are not counted as successful.
    pub checked: usize,
    pub failed: usize,
    pub not_applicable: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub steps: usize,
    pub claims: usize,
    pub source_facts: usize,
    pub rule_applications: usize,
    pub conditions: [ConditionReport; 9],
    pub failures: Vec<Failure>,
}
impl Report {
    fn new(certificate: &Certificate) -> Self {
        let source_facts = certificate
            .steps
            .iter()
            .filter(|step| matches!(step.justification, Justification::Source(_)))
            .count();
        Self {
            steps: certificate.steps.len(),
            claims: certificate.claims.len(),
            source_facts,
            rule_applications: certificate.steps.len() - source_facts,
            conditions: Condition::ALL.map(|condition| ConditionReport {
                condition,
                checked: 0,
                failed: 0,
                not_applicable: match condition {
                    Condition::C5 => Some("no interpreted built-ins"),
                    Condition::C6 => Some("no trust obligations or inference fuses"),
                    Condition::C8 => Some("no existential witnesses"),
                    _ => None,
                },
            }),
            failures: Vec::new(),
        }
    }

    pub fn valid(&self) -> bool {
        self.failures.is_empty()
    }
    pub fn condition(&self, condition: Condition) -> &ConditionReport {
        &self.conditions[condition as usize]
    }

    /// Turn an invalid report into a diagnostic error for CLI/browser callers.
    /// Library callers can inspect `valid()` and the structured failures instead.
    pub fn require_valid(self) -> Result<Self, Error> {
        if self.valid() {
            Ok(self)
        } else {
            Err(Error::new(self.to_string()))
        }
    }

    fn attempt(&mut self, condition: Condition) {
        self.conditions[condition as usize].checked += 1;
    }
    fn fail(&mut self, condition: Condition, location: ProofLocation, detail: impl Into<String>) {
        self.conditions[condition as usize].failed += 1;
        self.failures.push(Failure {
            condition,
            location,
            detail: detail.into(),
        });
    }

    fn reference(&mut self, id: usize, length: usize, kind: &str, location: ProofLocation) -> bool {
        self.attempt(Condition::C4);
        if id > 0 && id <= length {
            true
        } else {
            self.fail(Condition::C4, location, format!("Unknown {kind} {id}."));
            false
        }
    }
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let plural = |count| if count == 1 { "" } else { "s" };
        writeln!(
            f,
            "{}: {} step{}, {} claim{}",
            if self.valid() { "checked" } else { "invalid" },
            self.steps,
            plural(self.steps),
            self.claims,
            plural(self.claims)
        )?;
        writeln!(
            f,
            "{} source fact{}; {} rule application{}",
            self.source_facts,
            plural(self.source_facts),
            self.rule_applications,
            plural(self.rule_applications)
        )?;
        writeln!(f, "Conditions:")?;
        for result in &self.conditions {
            if let Some(reason) = result.not_applicable {
                writeln!(f, "  {}: not applicable ({reason})", result.condition)?;
            } else {
                writeln!(
                    f,
                    "  {}: {} checked, {} failed",
                    result.condition, result.checked, result.failed
                )?;
            }
        }
        if !self.failures.is_empty() {
            writeln!(f, "Failures:")?;
            for failure in &self.failures {
                writeln!(
                    f,
                    "  [{}] {}: {}",
                    failure.condition, failure.location, failure.detail
                )?;
            }
        }
        writeln!(
            f,
            "Source assertions are assumed. Answer completeness is not checked."
        )
    }
}

fn match_premises(goals: &[Goal], premises: &[usize], steps: &[Step]) -> Option<Bindings> {
    if goals.len() != premises.len() {
        return None;
    }
    let mut bindings = Bindings::new();
    for (goal, id) in goals.iter().zip(premises) {
        let step = steps.get(id.checked_sub(1)?)?;
        if !match_goal(goal, &step.atom, &mut bindings) {
            return None;
        }
    }
    Some(bindings)
}

/// Validate recorded applications without forward search. Semantic failures
/// produce an invalid report; malformed source programs return an error.
/// Callers must inspect `Report::valid()` or call `Report::require_valid()`.
pub fn check(program: &Program, certificate: &Certificate) -> Result<Report, Error> {
    use Condition::*;
    program.validate()?;
    let mut report = Report::new(certificate);
    let mut known = HashSet::new();
    for (index, step) in certificate.steps.iter().enumerate() {
        let location = ProofLocation::Step(index + 1);
        report.attempt(C3);
        if !known.insert(&step.atom) {
            report.fail(C3, location, "Duplicate conclusion.");
        }
        match &step.justification {
            Justification::Source(source) => {
                if report.reference(*source, program.facts.len(), "source fact", location) {
                    report.attempt(C1);
                    if program.facts[source - 1] != step.atom {
                        report.fail(
                            C1,
                            location,
                            "Conclusion differs from the cited source fact.",
                        );
                    }
                }
            }
            Justification::Rule { rule, premises } => {
                let mut covered = report.reference(*rule, program.rules.len(), "rule", location);
                let mut dependencies_covered = true;
                let mut forward = false;
                for premise in premises {
                    let exists = report.reference(
                        *premise,
                        certificate.steps.len(),
                        "premise step",
                        location,
                    );
                    covered &= exists;
                    dependencies_covered &= exists;
                    forward |= exists && *premise > index;
                }
                if dependencies_covered {
                    report.attempt(C2);
                    if forward {
                        report.fail(C2, location, "Every premise must refer to an earlier step; cycles and forward references are forbidden.");
                    }
                }
                if covered {
                    report.attempt(C1);
                    let clause = &program.rules[rule - 1];
                    if premises.len() != clause.body.len() {
                        report.fail(C1, location, "Premise count differs from the rule body.");
                    } else if let Some(bindings) =
                        match_premises(&clause.body, premises, &certificate.steps)
                    {
                        if instantiate(&clause.head, &bindings).as_ref() != Some(&step.atom) {
                            report.fail(
                                C1,
                                location,
                                "Conclusion is not this rule application's head.",
                            );
                        }
                    } else {
                        report.fail(
                            C1,
                            location,
                            "Premises do not match under one consistent substitution.",
                        );
                    }
                }
            }
        }
    }

    let source_atoms: HashSet<_> = program.facts.iter().collect();
    let mut answers: HashSet<(usize, Vec<Value>)> = HashSet::new();
    let mut derived = HashSet::new();
    for (index, claim) in certificate.claims.iter().enumerate() {
        let location = ProofLocation::Claim(index + 1);
        report.attempt(C3);
        match claim {
            Claim::Derived(id) => {
                report.attempt(C9);
                if !derived.insert(*id) {
                    report.fail(C3, location, "Duplicate derived claim.");
                }
                if !program.queries.is_empty() {
                    report.fail(
                        C9,
                        location,
                        "A program with queries must claim query selections, not derived steps.",
                    );
                }
                if report.reference(*id, certificate.steps.len(), "claimed step", location) {
                    report.attempt(C7);
                    let step = &certificate.steps[id - 1];
                    if !matches!(step.justification, Justification::Rule { .. })
                        || source_atoms.contains(&step.atom)
                    {
                        report.fail(C7, location, "A derived-output claim must be a rule conclusion absent from the source facts.");
                    }
                }
            }
            Claim::Answer { query, premises } => {
                let mut covered =
                    report.reference(*query, program.queries.len(), "query", location);
                for premise in premises {
                    covered &= report.reference(
                        *premise,
                        certificate.steps.len(),
                        "query premise step",
                        location,
                    );
                }
                if covered {
                    report.attempt(C9);
                    let source_query = &program.queries[query - 1];
                    if premises.len() != source_query.goals.len() {
                        report.fail(C9, location, "All query goals need evidence.");
                    } else if let Some(bindings) =
                        match_premises(&source_query.goals, premises, &certificate.steps)
                    {
                        let values = source_query
                            .variables
                            .iter()
                            .map(|name| bindings[name].clone())
                            .collect();
                        if !answers.insert((*query, values)) {
                            report.fail(C3, location, "Duplicate projected answer.");
                        }
                    } else {
                        report.fail(
                            C9,
                            location,
                            "Query goals do not match under one consistent substitution.",
                        );
                    }
                }
            }
        }
    }

    // Walk even malformed dependency graphs safely so independent C7 findings
    // remain available alongside missing-reference and cycle diagnostics.
    let mut relevant = HashSet::new();
    let mut pending: Vec<_> = certificate.claims.iter().flat_map(claim_premises).collect();
    while let Some(id) = pending.pop() {
        if let Some(step) = id.checked_sub(1).and_then(|id| certificate.steps.get(id)) {
            if relevant.insert(id) {
                if let Justification::Rule { premises, .. } = &step.justification {
                    pending.extend(premises);
                }
            }
        }
    }
    for index in 0..certificate.steps.len() {
        report.attempt(C7);
        if !relevant.contains(&(index + 1)) {
            report.fail(
                C7,
                ProofLocation::Step(index + 1),
                "Step is unrelated to every claim.",
            );
        }
    }
    Ok(report)
}
