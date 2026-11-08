use std::io::Write;
use std::process::ExitCode;

use data::column_name;

const USAGE: &str = "Usage: xls_column_names [N]

Print a TSV mapping from zero-based column indices to Excel column names
(0 -> A, 1 -> B, ..., 25 -> Z, 26 -> AA), one line per column, streamed to
stdout line by line.

Arguments:
  N  stop after N lines (default: unlimited)

Options:
  -h, --help  show this help
";

fn main() -> ExitCode {
    let mut limit: Option<usize> = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            s => match s.parse::<usize>() {
                Ok(n) if limit.is_none() => limit = Some(n),
                Ok(_) => {
                    eprintln!("unexpected extra argument: {s}\n{USAGE}");
                    return ExitCode::from(2);
                }
                Err(_) => {
                    eprintln!("invalid line count: {s:?}\n{USAGE}");
                    return ExitCode::from(2);
                }
            },
        }
    }

    let mut out = std::io::stdout().lock();
    let mut index: usize = 0;
    loop {
        if limit.is_some_and(|n| index >= n) {
            return ExitCode::SUCCESS;
        }
        // A closed downstream pipe (e.g. `| head`) is a normal exit, not an error.
        if writeln!(out, "{index}\t{}", column_name(index)).is_err() {
            return ExitCode::SUCCESS;
        }
        index += 1;
    }
}
