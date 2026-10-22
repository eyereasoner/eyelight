use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String);

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}

/// Values have structural identity: an atom, string, and integer are distinct.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Symbol(String),
    Text(String),
    Integer(i64),
}

fn quoted(f: &mut fmt::Formatter<'_>, value: &str, quote: char) -> fmt::Result {
    write!(f, "{quote}")?;
    for character in value.chars() {
        match character {
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            '\\' => f.write_str("\\\\")?,
            c if c == quote => write!(f, "\\{c}")?,
            c => write!(f, "{c}")?,
        }
    }
    write!(f, "{quote}")
}

pub(crate) fn symbol(f: &mut fmt::Formatter<'_>, value: &str) -> fmt::Result {
    let mut characters = value.chars();
    if characters.next().is_some_and(|c| c.is_ascii_lowercase())
        && characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        f.write_str(value)
    } else {
        quoted(f, value, '\'')
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbol(value) => symbol(f, value),
            Self::Text(value) => quoted(f, value, '"'),
            Self::Integer(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Atom {
    pub predicate: String,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Variable(String),
    Constant(Value),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Goal {
    pub predicate: String,
    pub terms: Vec<Term>,
}

impl Goal {
    pub fn variables(&self) -> BTreeSet<String> {
        self.terms
            .iter()
            .filter_map(|t| match t {
                Term::Variable(name) => Some(name.clone()),
                _ => None,
            })
            .collect()
    }
    pub fn ground(&self) -> Option<Atom> {
        Some(Atom {
            predicate: self.predicate.clone(),
            values: self
                .terms
                .iter()
                .map(|t| match t {
                    Term::Constant(value) => Some(value.clone()),
                    _ => None,
                })
                .collect::<Option<_>>()?,
        })
    }
}

fn relation(
    f: &mut fmt::Formatter<'_>,
    predicate: &str,
    values: &[impl fmt::Display],
) -> fmt::Result {
    symbol(f, predicate)?;
    if !values.is_empty() {
        f.write_str("(")?;
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{value}")?;
        }
        f.write_str(")")?;
    }
    Ok(())
}
impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(name) if name.starts_with("_$") => f.write_str("_"),
            Self::Variable(name) => f.write_str(name),
            Self::Constant(value) => write!(f, "{value}"),
        }
    }
}
impl fmt::Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        relation(f, &self.predicate, &self.terms)
    }
}
impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        relation(f, &self.predicate, &self.values)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub head: Goal,
    pub body: Vec<Goal>,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} :- ", self.head)?;
        for (index, goal) in self.body.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{goal}")?;
        }
        f.write_str(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub goals: Vec<Goal>,
    pub variables: Vec<String>,
}

/// Merge parsed documents with `extend`; variable scopes are local to clauses.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub facts: Vec<Atom>,
    pub rules: Vec<Rule>,
    pub queries: Vec<Query>,
}

impl Program {
    pub fn extend(&mut self, other: Self) {
        self.facts.extend(other.facts);
        self.rules.extend(other.rules);
        self.queries.extend(other.queries);
    }
    pub fn validate(&self) -> Result<(), Error> {
        for (index, rule) in self.rules.iter().enumerate() {
            if rule.body.is_empty() || rule.body.len() > 64 {
                return Err(Error::new(format!(
                    "Rule {} needs 1 to 64 body goals.",
                    index + 1
                )));
            }
            let bound: BTreeSet<_> = rule.body.iter().flat_map(Goal::variables).collect();
            if !rule.head.variables().is_subset(&bound) {
                return Err(Error::new(format!(
                    "Rule {} has a head variable absent from its body.",
                    index + 1
                )));
            }
        }
        for (index, query) in self.queries.iter().enumerate() {
            let bound: BTreeSet<_> = query.goals.iter().flat_map(Goal::variables).collect();
            let projected: BTreeSet<_> = query.variables.iter().cloned().collect();
            if query.goals.is_empty()
                || query.goals.len() > 64
                || projected.len() != query.variables.len()
                || !projected.is_subset(&bound)
            {
                return Err(Error::new(format!(
                    "Query {} has invalid goals or projected variables.",
                    index + 1
                )));
            }
        }
        Ok(())
    }
}
