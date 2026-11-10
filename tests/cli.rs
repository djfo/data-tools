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
fn fields_expands_column_range() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .args([fixture(), "-f", "A:B"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "Name\tAge\nAnna\t34\nBob\t28\n"
    );
}

#[test]
fn fields_accepts_mixed_ranges_and_names() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .args([fixture(), "-f", "C,A:B"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "City\tName\tAge\nWien\tAnna\t34\nBerlin\tBob\t28\n"
    );
}

#[test]
fn fields_rejects_reversed_range() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx"))
        .args([fixture(), "-f", "C:A"])
        .output()
        .unwrap();
    assert!(!out.status.success());
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

fn tsv_fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/table.tsv")
}

fn pipes_fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pipes.tsv")
}

#[test]
fn tsv_to_md_converts_with_alignment_spec() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-to-md"))
        .args([tsv_fixture(), "-a", "llrc"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "| Name | Age | Score |  City  |\n\
         | :--- | :-- | ----: | :----: |\n\
         | Anna | 34  |  91.5 |  Wien  |\n\
         | Bob  | 28  |    77 | Berlin |\n"
    );
}

#[test]
fn tsv_to_md_without_align_uses_default() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-to-md"))
        .arg(tsv_fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "| Name | Age | Score | City   |\n\
         | ---- | --- | ----- | ------ |\n\
         | Anna | 34  | 91.5  | Wien   |\n\
         | Bob  | 28  | 77    | Berlin |\n"
    );
}

#[test]
fn tsv_to_md_accepts_joined_align_flag() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-to-md"))
        .args(["-allrc", tsv_fixture()])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout)
        .unwrap()
        .starts_with("| Name | Age | Score |"));
}

#[test]
fn tsv_to_md_escapes_pipes_in_cells() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-to-md"))
        .arg(pipes_fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "| a\\|b | c   |\n| ---- | --- |\n| d\\|e | f   |\n"
    );
}

#[test]
fn tsv_to_md_rejects_invalid_alignment_char() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-to-md"))
        .args([tsv_fixture(), "-a", "lrx"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn tsv_to_md_rejects_more_alignments_than_columns() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-to-md"))
        .args([tsv_fixture(), "-a", "lllrc"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn tsv_to_md_missing_file_fails() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-to-md"))
        .arg("does-not-exist.tsv")
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn tsv_to_md_no_arguments_shows_usage() {
    let out = Command::new(env!("CARGO_BIN_EXE_tsv-to-md"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("Usage:"));
}
