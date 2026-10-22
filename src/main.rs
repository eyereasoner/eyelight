use eyelight::{check_with_limits, parse, reason, Certificate, Error, Limits, Program, VERSION};
use std::io::{self, Read};
use std::process::ExitCode;

const HELP: &str = "eyelight — a small relational reasoner

Usage: eyelight [OPTIONS] FILE...
       eyelight [OPTIONS] -

  -e, --explain              Answers with their supporting rules and facts
  -p, --proof                Write a checkable proof certificate
  -c, --prove-and-check      Generate and check a certificate
      --check-proof FILE    Check a saved certificate against the source files
      --max-facts N         Maximum stored fact/evidence steps (default 1000000)
      --max-matches N       Maximum matching/work operations (default 10000000)
  -h, --help                Show this help
  -v, --version             Show the version

Files use Prolog-style .pl syntax: facts, Head :- Body, and ?- queries.
Use - for stdin. Multiple files form one program, in the supplied order.
Proof checking verifies recorded claims, not answer completeness.
";

fn read(path: &str) -> Result<String, Error> {
    if path == "-" {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .map_err(|error| Error::new(format!("Cannot read stdin: {error}")))?;
        Ok(text)
    } else {
        std::fs::read_to_string(path)
            .map_err(|error| Error::new(format!("Cannot read {path:?}: {error}")))
    }
}

fn run() -> Result<String, Error> {
    let mut args = std::env::args().skip(1);
    let mut files = Vec::new();
    let mut mode = None;
    let mut proof = None;
    let mut limits = Limits::default();
    let mut positional = false;
    while let Some(arg) = args.next() {
        if positional {
            files.push(arg);
            continue;
        }
        let selected = match arg.as_str() {
            "--" => {
                positional = true;
                continue;
            }
            "-h" | "--help" => return Ok(HELP.to_owned()),
            "-v" | "--version" => return Ok(format!("eyelight {VERSION}\n")),
            "-e" | "--explain" => Some("explain"),
            "-p" | "--proof" => Some("proof"),
            "-c" | "--prove-and-check" => Some("checked"),
            "--check-proof" => {
                proof = Some(
                    args.next()
                        .ok_or_else(|| Error::new("--check-proof needs a file name or -."))?,
                );
                Some("check")
            }
            "--max-facts" | "--max-matches" => {
                let value = args
                    .next()
                    .and_then(|text| text.parse::<usize>().ok())
                    .filter(|n| *n > 0)
                    .ok_or_else(|| Error::new(format!("{arg} needs a positive integer.")))?;
                if arg == "--max-facts" {
                    limits.max_facts = value;
                } else {
                    limits.max_matches = value;
                }
                None
            }
            "-" => {
                files.push(arg);
                None
            }
            value if value.starts_with('-') => {
                return Err(Error::new(format!("Unknown option {value:?}; use --help.")))
            }
            _ => {
                files.push(arg);
                None
            }
        };
        if let Some(selected) = selected {
            if mode.replace(selected).is_some() {
                return Err(Error::new("Choose one output mode: --explain, --proof, --prove-and-check, or --check-proof."));
            }
        }
    }
    if files.is_empty() {
        if mode.is_none() {
            return Ok(HELP.to_owned());
        }
        return Err(Error::new("Supply a source file or - for stdin."));
    }
    if files.iter().filter(|file| file.as_str() == "-").count()
        + usize::from(proof.as_deref() == Some("-"))
        > 1
    {
        return Err(Error::new("Only one input may read stdin."));
    }
    let mut program = Program::default();
    for file in files {
        let parsed =
            parse(&read(&file)?).map_err(|error| Error::new(format!("{file}: {error}")))?;
        program.extend(parsed);
    }
    let mode = mode.unwrap_or("answers");
    if let Some(file) = proof {
        return Ok(
            check_with_limits(&program, &Certificate::parse(&read(&file)?)?, limits)?
                .require_valid()?
                .to_string(),
        );
    }
    let result = reason(&program, limits)?;
    match mode {
        "answers" => Ok(result.output(&program)),
        "explain" => Ok(result.explain(&program)),
        "proof" => Ok(result.certificate(&program).to_string()),
        "checked" => Ok(
            check_with_limits(&program, &result.certificate(&program), limits)?
                .require_valid()?
                .to_string(),
        ),
        _ => unreachable!(),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("eyelight: {error}");
            ExitCode::FAILURE
        }
    }
}
