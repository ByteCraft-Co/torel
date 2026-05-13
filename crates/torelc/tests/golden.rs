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
fn checks_valid_call_no_args_fixture() {
    let output = run_torelc(&["tests/fixtures/valid/call_no_args.torel", "--emit", "check"]);

    assert!(output.status.success(), "valid call fixture should pass");
}

#[test]
fn checks_valid_call_with_arg_fixture() {
    let output = run_torelc(&[
        "tests/fixtures/valid/call_with_arg.torel",
        "--emit",
        "check",
    ]);

    assert!(output.status.success(), "valid call fixture should pass");
}

#[test]
fn checks_valid_return_param_fixture() {
    let output = run_torelc(&["tests/fixtures/valid/return_param.torel", "--emit", "check"]);

    assert!(
        output.status.success(),
        "valid return param fixture should pass"
    );
}

#[test]
fn checks_valid_return_int_literal_fixture() {
    let output = run_torelc(&[
        "tests/fixtures/valid/return_int_literal.torel",
        "--emit",
        "check",
    ]);

    assert!(output.status.success(), "valid int literal should pass");
}

#[test]
fn checks_valid_return_text_literal_fixture() {
    let output = run_torelc(&[
        "tests/fixtures/valid/return_text_literal.torel",
        "--emit",
        "check",
    ]);

    assert!(output.status.success(), "valid text literal should pass");
}

#[test]
fn checks_valid_return_bool_literal_fixture() {
    let output = run_torelc(&[
        "tests/fixtures/valid/return_bool_literal.torel",
        "--emit",
        "check",
    ]);

    assert!(output.status.success(), "valid bool literal should pass");
}

#[test]
fn checks_valid_fix_int_return_fixture() {
    let output = run_torelc(&[
        "tests/fixtures/valid/fix_int_return.torel",
        "--emit",
        "check",
    ]);

    assert!(output.status.success(), "valid fix return should pass");
}

#[test]
fn checks_valid_fix_call_arg_fixture() {
    let output = run_torelc(&["tests/fixtures/valid/fix_call_arg.torel", "--emit", "check"]);

    assert!(output.status.success(), "valid fix call arg should pass");
}

#[test]
fn checks_valid_slot_assign_int_fixture() {
    let output = run_torelc(&[
        "tests/fixtures/valid/slot_assign_int.torel",
        "--emit",
        "check",
    ]);

    assert!(
        output.status.success(),
        "valid int slot assignment should pass"
    );
}

#[test]
fn checks_valid_slot_assign_bool_fixture() {
    let output = run_torelc(&[
        "tests/fixtures/valid/slot_assign_bool.torel",
        "--emit",
        "check",
    ]);

    assert!(
        output.status.success(),
        "valid bool slot assignment should pass"
    );
}

#[test]
fn checks_valid_slot_assign_from_call_fixture() {
    let output = run_torelc(&[
        "tests/fixtures/valid/slot_assign_from_call.torel",
        "--emit",
        "check",
    ]);

    assert!(
        output.status.success(),
        "valid slot assignment from call should pass"
    );
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

#[test]
fn rejects_unknown_proc_call() {
    assert_failure(
        "tests/fixtures/invalid/unknown_proc_call.torel",
        "error: unknown procedure `nope`\n",
    );
}

#[test]
fn rejects_arg_count_mismatch() {
    assert_failure(
        "tests/fixtures/invalid/arg_count_mismatch.torel",
        "error: argument count mismatch for `id_exit`: expected 1, found 0\n",
    );
}

#[test]
fn rejects_arg_type_mismatch() {
    assert_failure(
        "tests/fixtures/invalid/arg_type_mismatch.torel",
        "error: argument type mismatch for `id_i32` argument 1: expected `Int32`, found `Exit`\n",
    );
}

#[test]
fn rejects_proc_used_as_value() {
    assert_failure(
        "tests/fixtures/invalid/proc_used_as_value.torel",
        "error: procedure `make_exit` used as value\n",
    );
}

#[test]
fn rejects_not_callable() {
    assert_failure(
        "tests/fixtures/invalid/not_callable.torel",
        "error: `Exit.ok` is not callable\n",
    );
}

#[test]
fn rejects_unknown_local() {
    assert_failure(
        "tests/fixtures/invalid/unknown_local.torel",
        "error: unknown local `answer`\n",
    );
}

#[test]
fn rejects_duplicate_local() {
    assert_failure(
        "tests/fixtures/invalid/duplicate_local.torel",
        "error: duplicate local `answer`\n",
    );
}

#[test]
fn rejects_fix_type_mismatch() {
    assert_failure(
        "tests/fixtures/invalid/fix_type_mismatch.torel",
        "error: local `answer` type mismatch: expected `Int32`, found `Exit`\n",
    );
}

#[test]
fn rejects_return_literal_mismatch() {
    assert_failure(
        "tests/fixtures/invalid/return_literal_mismatch.torel",
        "error: return type mismatch: expected `Exit`, found `Int32`\n",
    );
}

#[test]
fn rejects_assign_to_fix() {
    assert_failure(
        "tests/fixtures/invalid/assign_to_fix.torel",
        "error: cannot assign to immutable local `answer`\n",
    );
}

#[test]
fn rejects_assign_unknown_local() {
    assert_failure(
        "tests/fixtures/invalid/assign_unknown_local.torel",
        "error: unknown local `answer`\n",
    );
}

#[test]
fn rejects_assign_type_mismatch() {
    assert_failure(
        "tests/fixtures/invalid/assign_type_mismatch.torel",
        "error: assignment to `answer` type mismatch: expected `Int32`, found `Exit`\n",
    );
}

#[test]
fn rejects_assign_to_value_path() {
    assert_failure(
        "tests/fixtures/invalid/assign_to_value_path.torel",
        "error: invalid assignment target `Exit.ok`\n",
    );
}
