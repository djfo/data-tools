use std::io::{BufRead, Write};
use std::process::ExitCode;

use data::{find_column, tsv_to_dot, Error};

const USAGE: &str = "Usage: tsv-graph <file.tsv> <ID-column> <target-column>\n\nRead a TSV file and emit a directed graph in DOT format on stdout.\nEach data row produces one edge from its ID to its target column.\nThe first row of the TSV is treated as the header.\n\nOptions:\n  -h, --help  show this help\n";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut file: Option<String> = None;
    let mut id_col: Option<String> = None;
    let mut target_col: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            s if s.starts_with('-') => {
                eprintln!("unknown option: {s}\n{USAGE}");
                return ExitCode::from(2);
            }
            s => {
                if file.is_none() {
                    file = Some(s.to_string());
                } else if id_col.is_none() {
                    id_col = Some(s.to_string());
                } else if target_col.is_none() {
                    target_col = Some(s.to_string());
                } else {
                    eprintln!("unexpected extra argument: {s}\n{USAGE}");
                    return ExitCode::from(2);
                }
            }
        }
    }
    let (Some(file), Some(id_col), Some(target_col)) = (file, id_col, target_col) else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };

    match run(&file, &id_col, &target_col) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("tsv-graph: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(file: &str, id_name: &str, target_name: &str) -> Result<(), Error> {
    let path = std::path::Path::new(file);
    let f = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(f);
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        rows.push(line.split('\t').map(str::to_string).collect());
    }
    let Some(header) = rows.first() else {
        return Err(Error::Spreadsheet("empty TSV file".to_string()));
    };
    let id_col = find_column(header, id_name)
        .ok_or_else(|| Error::Spreadsheet(format!("column not found: {id_name:?}")))?;
    let target_col = find_column(header, target_name)
        .ok_or_else(|| Error::Spreadsheet(format!("column not found: {target_name:?}")))?;
    let mut out = std::io::stdout().lock();
    out.write_all(tsv_to_dot(&rows, id_col, target_col).as_bytes())?;
    Ok(())
}
