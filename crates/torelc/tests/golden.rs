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

fn assert_emit_matches_golden(emit: &str, expected: &str) {
    let output = run_torelc(&["examples/hello.torel", "--emit", emit]);

    assert_eq!(stdout(output), expected);
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
fn rejects_trailing_junk_fixture() {
    let output = run_torelc(&["tests/fixtures/trailing_junk.torel", "--emit", "check"]);

    assert!(!output.status.success(), "junk fixture should fail");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("expected top-level item"),
        "stderr should explain the parser failure:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("at 30..31"),
        "stderr should include the junk token span:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
