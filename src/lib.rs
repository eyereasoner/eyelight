//! Stratified relational reasoning, with a separate certificate checker.
//!
//! Parse a program with [`parse`], evaluate it with [`reason`], and inspect
//! answers or save a [`Certificate`]. Checking replays recorded applications;
//! it does not run the evaluator or establish answer completeness.
//!
//! ```
//! use eyelight::{check, parse, reason, Limits};
//! let program = parse("human(socrates). mortal(X) :- human(X). ?- mortal(Who).")?;
//! let run = reason(&program, Limits::default())?;
//! let report = check(&program, &run.certificate(&program))?;
//! assert!(report.valid());
//! assert_eq!(report.claims, 1);
//! # Ok::<(), eyelight::Error>(())
//! ```

mod engine;
mod language;
mod model;
mod parser;
mod proof;
mod semantics;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use engine::{reason, Answer, Limits, Run};
pub use model::{
    Aggregate, AggregateKind, Atom, Error, Goal, Literal, Program, Query, Rule, Term, Value,
};
pub use parser::{parse, parse_atom};
pub use proof::{
    check, check_with_limits, Boundary, Certificate, Claim, Condition, ConditionReport, Failure,
    Justification, Owner, ProofLocation, Report, Step,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run the same small interface in the CLI and browser.
pub fn execute(
    source: &str,
    mode: &str,
    saved_proof: &str,
    limits: Limits,
) -> Result<String, Error> {
    let program = parse(source)?;
    if mode == "check" {
        let certificate = Certificate::parse(saved_proof)?;
        return Ok(check_with_limits(&program, &certificate, limits)?
            .require_valid()?
            .to_string());
    }
    let run = reason(&program, limits)?;
    match mode {
        "answers" => Ok(run.output(&program)),
        "explain" => Ok(run.explain(&program)),
        "proof" => Ok(run.certificate(&program).to_string()),
        "checked" => Ok(
            check_with_limits(&program, &run.certificate(&program), limits)?
                .require_valid()?
                .to_string(),
        ),
        _ => Err(Error::new(format!("Unknown mode {mode:?}."))),
    }
}
