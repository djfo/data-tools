use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use data::{parse_columns, read_first_sheet, to_tsv, Error};

const USAGE: &str = "Usage: xlsx <file.xlsx> [-f A,C,E] [-e]

Convert the first worksheet of an Excel document to TSV on stdout.

Options:
  -f, --fields <LIST>  comma-separated Excel column letters to keep, in the
                       given order (e.g. -f A,C,E)
  -e, --escape         backslash-escape backslash, tab, CR and LF in fields
                       (as \\\\, \\t, \\r, \\n) so each record stays on one line
  -h, --help           show this help
";

fn main() -> ExitCode {
    let mut file: Option<String> = None;
    let mut fields: Option<String> = None;
    let mut escape = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "-e" | "--escape" => escape = true,
            "-f" | "--fields" => match args.next() {
                Some(value) => fields = Some(value),
                None => {
                    eprint!("{USAGE}");
                    return ExitCode::from(2);
                }
            },
            s if s.starts_with("-f") && s.len() > 2 => fields = Some(s[2..].to_string()),
            s if s.starts_with('-') => {
                eprintln!("unknown option: {s}\n{USAGE}");
                return ExitCode::from(2);
            }
            s => {
                if file.is_some() {
                    eprintln!("unexpected extra argument: {s}\n{USAGE}");
                    return ExitCode::from(2);
                }
                file = Some(s.to_string());
            }
        }
    }
    let Some(file) = file else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };

    match run(&file, fields.as_deref(), escape) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xlsx: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(file: &str, fields: Option<&str>, escape: bool) -> Result<(), Error> {
    let columns = match fields {
        Some(spec) => Some(parse_columns(spec)?),
        None => None,
    };
    let rows = read_first_sheet(Path::new(file))?;
    let mut out = std::io::stdout().lock();
    out.write_all(to_tsv(&rows, columns.as_deref(), escape).as_bytes())?;
    Ok(())
}
