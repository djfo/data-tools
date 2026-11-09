use std::process::Command;

fn fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.xlsx")
}

fn multiline_fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/multiline.xlsx")
}

#[test]
fn converts_first_sheet_to_tsv() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "Name\tAge\tCity\nAnna\t34\tWien\nBob\t28\tBerlin\n"
    );
}

#[test]
fn fields_selects_and_reorders_columns() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .args([fixture(), "-f", "C,A"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "City\tName\nWien\tAnna\nBerlin\tBob\n"
    );
}

#[test]
fn fields_rejects_invalid_column() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .args([fixture(), "-f", "A,1"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn missing_file_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .arg("does-not-exist.xlsx")
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn escape_flag_keeps_one_record_per_line() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .args([multiline_fixture(), "-e"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "Text\nline1\\nline2\na\\tb\\\\c\n"
    );
}

#[test]
fn without_escape_specials_pass_through_raw() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .arg(multiline_fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "Text\nline1\nline2\na\tb\\c\n"
    );
}

#[test]
fn column_names_with_limit() {
    let out = Command::new(env!("CARGO_BIN_EXE_xls_column_names"))
        .arg("5")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "0\tA\n1\tB\n2\tC\n3\tD\n4\tE\n"
    );
}

#[test]
fn column_names_rejects_invalid_count() {
    let out = Command::new(env!("CARGO_BIN_EXE_xls_column_names"))
        .arg("abc")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn column_names_streams_without_limit() {
    use std::io::{BufRead, BufReader};
    let mut child = Command::new(env!("CARGO_BIN_EXE_xls_column_names"))
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();
    assert_eq!(lines.next().unwrap().unwrap(), "0\tA");
    for _ in 1..26 {
        lines.next().unwrap().unwrap();
    }
    assert_eq!(lines.next().unwrap().unwrap(), "26\tAA");
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn no_arguments_shows_usage() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx")).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("Usage:"));
}

fn graph_fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/graph.tsv")
}

#[test]
fn tsv_graph_emits_dot_edges_to_target() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-graph"))
        .args([graph_fixture(), "ID", "TargetID"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "digraph {\n  \"2\" -> \"1\";\n  \"3\" -> \"1\";\n  \"4\" -> \"2\";\n}\n"
    );
}

#[test]
fn tsv_graph_missing_column_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-graph"))
        .args([graph_fixture(), "ID", "NoSuchColumn"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn tsv_graph_missing_file_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-graph"))
        .args(["does-not-exist.tsv", "ID", "TargetID"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn tsv_graph_no_arguments_shows_usage() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-graph")).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("Usage:"));
}

#[test]
fn tsv_graph_node_label_emits_node_statements() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-graph"))
        .args([graph_fixture(), "ID", "TargetID", "-n", "Name"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "digraph {\n  \"1\" [label=\"Root\"];\n  \"2\" [label=\"Child A\"];\n  \"3\" [label=\"Child B\"];\n  \"4\" [label=\"Grandchild\"];\n  \"2\" -> \"1\";\n  \"3\" -> \"1\";\n  \"4\" -> \"2\";\n}\n"
    );
}

#[test]
fn tsv_graph_unknown_label_column_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-graph"))
        .args([graph_fixture(), "ID", "TargetID", "-n", "NoSuchColumn"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
