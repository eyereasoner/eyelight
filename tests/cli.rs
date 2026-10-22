use std::io::Write;
use std::process::{Command, Stdio};

fn cli(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_eyelight"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn stdin_answers_explanation_and_generated_proof_check() {
    let source = "human(socrates). mortal(X) :- human(X). ?- mortal(Who).";
    let result = cli(&["-"], source);
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "?- mortal(Who).\nWho = socrates.\n"
    );
    let result = cli(&["-e", "-"], source);
    assert!(String::from_utf8(result.stdout)
        .unwrap()
        .contains("rule 1; uses [1]"));
    let result = cli(&["-c", "-"], source);
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("checked: 2 steps, 1 claim"));
    for condition in eyelight::Condition::ALL {
        assert!(text.contains(&format!("{condition}:")));
    }
    assert!(text.contains("C5 re-decision: not applicable"));
}
#[test]
fn saved_proof_from_stdin_is_checked_against_source() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/socrates.eye");
    let result = cli(&["-p", source], "");
    assert!(result.status.success());
    let proof = String::from_utf8(result.stdout).unwrap();
    assert!(cli(&["--check-proof", "-", source], &proof)
        .status
        .success());
    let forged = proof.replace("mortal(socrates)", "mortal(plato)");
    let result = cli(&["--check-proof", "-", source], &forged);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let diagnostic = String::from_utf8(result.stderr).unwrap();
    assert!(diagnostic.contains("[C1 resolution] step 2:"));
    for condition in eyelight::Condition::ALL {
        assert!(diagnostic.contains(&format!("{condition}:")));
    }
}
#[test]
fn errors_and_limits_have_nonzero_status_and_no_partial_output() {
    for (args, input) in [
        (vec!["-"], "p(X)."),
        (vec!["--max-facts", "1", "-"], "p(a). q(X) :- p(X)."),
        (vec!["--bogus"], ""),
        (vec!["--check-proof", "-", "-"], ""),
        (vec!["-e", "-p", "-"], ""),
    ] {
        let result = cli(&args, input);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}
#[test]
fn help_version_and_multiple_source_files_work() {
    assert!(cli(&[], "").status.success());
    assert_eq!(
        String::from_utf8(cli(&["--version"], "").stdout).unwrap(),
        format!("eyelight {}\n", eyelight::VERSION)
    );
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/socrates.eye");
    let result = cli(&[path, "-"], "human(hypatia).");
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout)
        .unwrap()
        .contains("Who = hypatia."));
}
