use std::process::Command;

fn fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.xlsx")
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
fn no_arguments_shows_usage() {
    let out = Command::new(env!("CARGO_BIN_EXE_xlsx")).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("Usage:"));
}
