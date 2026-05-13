use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}

fn run_torelc(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_torelc"))
        .current_dir(workspace_root())
        .args(args)
        .output()
        .expect("torelc should run")
}

fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "torelc failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("stdout should be utf-8")
        .replace("\r\n", "\n")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone())
        .expect("stderr should be utf-8")
        .replace("\r\n", "\n")
}

fn assert_emit_matches_golden(emit: &str, expected: &str) {
    let output = run_torelc(&["examples/hello.torel", "--emit", emit]);

    assert_eq!(stdout(output), expected);
}

fn assert_failure(path: &str, expected: &str) {
    let output = run_torelc(&[path, "--emit", "check"]);

    assert!(!output.status.success(), "{path} should fail");
    assert_eq!(stderr(&output), expected);
}

#[test]
fn emits_tokens_golden() {
    assert_emit_matches_golden(
        "tokens",
        include_str!("../../../tests/golden/hello.tokens.txt"),
    );
}

#[test]
fn emits_ast_golden() {
    assert_emit_matches_golden("ast", include_str!("../../../tests/golden/hello.ast.txt"));
}

#[test]
fn emits_hir_golden() {
    assert_emit_matches_golden("hir", include_str!("../../../tests/golden/hello.hir.txt"));
}

#[test]
fn emits_check_golden() {
    assert_emit_matches_golden(
        "check",
        include_str!("../../../tests/golden/hello.check.txt"),
    );
}

#[test]
fn checks_valid_fixture() {
    let output = run_torelc(&["tests/fixtures/valid/hello.torel", "--emit", "check"]);

    assert!(output.status.success(), "valid fixture should pass");
}

#[test]
fn rejects_trailing_junk_fixture() {
    assert_failure(
        "tests/fixtures/invalid/trailing_junk.torel",
        "error: expected top-level item at 30..31\n",
    );
}

#[test]
fn rejects_unknown_return_type() {
    assert_failure(
        "tests/fixtures/invalid/unknown_return_type.torel",
        "error: unknown type `Potato`\n",
    );
}

#[test]
fn rejects_unknown_return_value() {
    assert_failure(
        "tests/fixtures/invalid/unknown_return_value.torel",
        "error: unknown value path `Exit.nope`\n",
    );
}

#[test]
fn rejects_bad_return_type() {
    assert_failure(
        "tests/fixtures/invalid/bad_return_type.torel",
        "error: return type mismatch: expected `Int32`, found `Exit`\n",
    );
}

#[test]
fn rejects_missing_return() {
    assert_failure(
        "tests/fixtures/invalid/missing_return.torel",
        "error: missing return from procedure `main`: expected `Exit`\n",
    );
}
