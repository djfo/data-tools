use std::io::{Read, Write};
use std::process::ExitCode;

use data::{parse_alignment, to_markdown, Error};

const USAGE: &str = "Usage: tsv-to-md <file.tsv> [-a llrc]

Convert a TSV file to a Markdown table on stdout. The first row is the
header row.

Options:
  -a, --align <SPEC>  per-column alignments, one character per column:
                      l = left, r = right, c = center, - = default;
                      columns beyond the spec are default-aligned
                      (e.g. -a llrc = left, left, right, center)
  -h, --help          show this help
";

fn main() -> ExitCode {
    let mut file: Option<String> = None;
    let mut align: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "-a" | "--align" => match args.next() {
                Some(value) => align = Some(value),
                None => {
                    eprint!("{USAGE}");
                    return ExitCode::from(2);
                }
            },
            s if s.starts_with("-a") && s.len() > 2 => align = Some(s[2..].to_string()),
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

    match run(&file, align.as_deref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("tsv-to-md: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(file: &str, align: Option<&str>) -> Result<(), Error> {
    let mut content = String::new();
    std::fs::File::open(file)?.read_to_string(&mut content)?;
    let rows: Vec<Vec<String>> = content
        .lines()
        .map(|line| line.split('\t').map(str::to_string).collect())
        .collect();
    let alignments = match align {
        Some(spec) => parse_alignment(spec)?,
        None => Vec::new(),
    };
    let ncols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if alignments.len() > ncols {
        return Err(Error::InvalidAlignment(format!(
            "spec has {} entries but the table has {ncols} column(s)",
            alignments.len()
        )));
    }
    let mut out = std::io::stdout().lock();
    out.write_all(to_markdown(&rows, &alignments).as_bytes())?;
    Ok(())
}
