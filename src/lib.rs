//! Core logic for the `data` command-line tools.

use std::fmt;
use std::path::Path;

use calamine::{open_workbook, Reader};

/// Errors produced by the conversion.
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Spreadsheet(String),
    InvalidColumn(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Spreadsheet(m) => write!(f, "spreadsheet error: {m}"),
            Error::InvalidColumn(c) => write!(f, "invalid column reference: {c:?}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Convert an Excel column reference ("A", "b", "AA") to a zero-based index.
pub fn column_index(reference: &str) -> Option<usize> {
    if reference.is_empty() {
        return None;
    }
    let mut index: usize = 0;
    for c in reference.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        index = index
            .checked_mul(26)?
            .checked_add(c.to_ascii_uppercase() as usize - 'A' as usize + 1)?;
    }
    Some(index - 1)
}

/// Convert a zero-based column index to its Excel column name.
pub fn column_name(index: usize) -> String {
    let mut name = String::new();
    let mut n = index + 1;
    while n > 0 {
        name.insert(0, (b'A' + ((n - 1) % 26) as u8) as char);
        n = (n - 1) / 26;
    }
    name
}

/// Parse one comma-separated item of a column spec: a single column ("A", "b")
/// or an inclusive range ("A:D", "AA:AZ"), expanded to zero-based indices.
/// The colon matches Excel's own range operator (e.g. `A:D`).
fn parse_column_item(item: &str) -> Result<Vec<usize>, Error> {
    let invalid = || Error::InvalidColumn(item.to_string());
    let (start_ref, end_ref) = if let Some((a, b)) = item.split_once(':') {
        (a, b)
    } else {
        return Ok(vec![
            column_index(item).ok_or_else(invalid)?,
        ]);
    };
    let start = column_index(start_ref).ok_or_else(invalid)?;
    let end = column_index(end_ref).ok_or_else(invalid)?;
    if start > end {
        return Err(invalid());
    }
    Ok((start..=end).collect())
}

/// Parse a comma-separated list of Excel column references and ranges
/// (e.g. "A:D,P,Q,AA:AZ") into zero-based indices, in the given order.
pub fn parse_columns(spec: &str) -> Result<Vec<usize>, Error> {
    let mut columns = Vec::new();
    for item in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        columns.extend(parse_column_item(item)?);
    }
    Ok(columns)
}

/// Read the first worksheet of the Excel document at `path` as a grid of strings.
pub fn read_first_sheet(path: &Path) -> Result<Vec<Vec<String>>, Error> {
    let mut workbook: calamine::Xlsx<std::io::BufReader<std::fs::File>> =
        open_workbook(path).map_err(|e: calamine::XlsxError| Error::Spreadsheet(e.to_string()))?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| Error::Spreadsheet("workbook has no worksheets".to_string()))?
        .map_err(|e| Error::Spreadsheet(e.to_string()))?;
    Ok(range
        .rows()
        .map(|row| row.iter().map(|cell| cell.to_string()).collect())
        .collect())
}

/// Backslash-escape a field so it contains no raw backslash, tab, CR or LF.
pub fn escape_cell(cell: &str) -> String {
    let mut out = String::with_capacity(cell.len());
    for c in cell.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

/// Render a grid as TSV, keeping only `columns` (zero-based) when given.
/// When `escape` is set, fields are backslash-escaped via [`escape_cell`].
pub fn to_tsv(rows: &[Vec<String>], columns: Option<&[usize]>, escape: bool) -> String {
    let mut out = String::new();
    for row in rows {
        let cells: Vec<&str> = match columns {
            None => row.iter().map(String::as_str).collect(),
            Some(cols) => cols
                .iter()
                .map(|&i| row.get(i).map(String::as_str).unwrap_or(""))
                .collect(),
        };
        let mut first = true;
        for cell in cells {
            if !first {
                out.push('\t');
            }
            first = false;
            if escape {
                out.push_str(&escape_cell(cell));
            } else {
                out.push_str(cell);
            }
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_index_single_letters() {
        assert_eq!(column_index("A"), Some(0));
        assert_eq!(column_index("B"), Some(1));
        assert_eq!(column_index("Z"), Some(25));
        assert_eq!(column_index("a"), Some(0));
    }

    #[test]
    fn column_index_multi_letters() {
        assert_eq!(column_index("AA"), Some(26));
        assert_eq!(column_index("AB"), Some(27));
        assert_eq!(column_index("AZ"), Some(51));
        assert_eq!(column_index("BA"), Some(52));
    }

    #[test]
    fn column_index_rejects_invalid() {
        assert_eq!(column_index(""), None);
        assert_eq!(column_index("1"), None);
        assert_eq!(column_index("A1"), None);
        assert_eq!(column_index(" A"), None);
    }

    #[test]
    fn column_name_single_and_multi_letters() {
        assert_eq!(column_name(0), "A");
        assert_eq!(column_name(25), "Z");
        assert_eq!(column_name(26), "AA");
        assert_eq!(column_name(27), "AB");
        assert_eq!(column_name(51), "AZ");
        assert_eq!(column_name(52), "BA");
    }

    #[test]
    fn column_name_is_inverse_of_column_index() {
        for s in ["A", "B", "Z", "AA", "AZ", "BA", "ZZ", "AAA"] {
            assert_eq!(column_name(column_index(s).unwrap()), s);
        }
    }

    #[test]
    fn parse_columns_splits_and_trims() {
        assert_eq!(parse_columns("A, C ,e").unwrap(), vec![0, 2, 4]);
        assert!(parse_columns("A,,B").is_ok());
        assert!(parse_columns("A,x1").is_err());
    }

    #[test]
    fn parse_columns_expands_colon_range() {
        assert_eq!(parse_columns("A:D").unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(parse_columns("B:C").unwrap(), vec![1, 2]);
        assert_eq!(parse_columns("Y:AA").unwrap(), vec![24, 25, 26]);
        assert_eq!(parse_columns("AA:AZ").unwrap(), (26..=51).collect::<Vec<_>>());
        assert_eq!(parse_columns("A:A").unwrap(), vec![0]);
    }

    #[test]
    fn parse_columns_mixes_ranges_and_names() {
        assert_eq!(
            parse_columns("A:D,P,Q,AA:AZ").unwrap(),
            [vec![0, 1, 2, 3], vec![15, 16], (26..=51).collect::<Vec<_>>()].concat()
        );
        assert_eq!(parse_columns("C,A:B").unwrap(), vec![2, 0, 1]);
    }

    #[test]
    fn parse_columns_rejects_invalid_ranges() {
        assert!(parse_columns("D:A").is_err());
        assert!(parse_columns("A:").is_err());
        assert!(parse_columns(":D").is_err());
        assert!(parse_columns("A:1").is_err());
        assert!(parse_columns("A:B:C").is_err());
        assert!(parse_columns("A:B,x1").is_err());
        assert!(parse_columns("A..D").is_err());
        assert!(parse_columns("AA-AZ").is_err());
    }

    #[test]
    fn escape_cell_escapes_specials() {
        assert_eq!(escape_cell("a\tb\\c\nd\r"), "a\\tb\\\\c\\nd\\r");
        assert_eq!(escape_cell("plain"), "plain");
    }

    #[test]
    fn to_tsv_without_selection() {
        let rows = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["1".to_string(), "2".to_string()],
        ];
        assert_eq!(to_tsv(&rows, None, false), "a\tb\n1\t2\n");
    }

    #[test]
    fn to_tsv_with_selection_reorders() {
        let rows = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
        ];
        assert_eq!(to_tsv(&rows, Some(&[2, 0]), false), "c\ta\n3\t1\n");
    }

    #[test]
    fn to_tsv_missing_column_is_empty() {
        let rows = vec![vec!["a".to_string()]];
        assert_eq!(to_tsv(&rows, Some(&[0, 5]), false), "a\t\n");
    }

    #[test]
    fn to_tsv_escape_keeps_one_record_per_line() {
        let rows = vec![vec!["x\ny".to_string(), "a\tb".to_string()]];
        assert_eq!(to_tsv(&rows, None, true), "x\\ny\ta\\tb\n");
        assert_eq!(to_tsv(&rows, None, false), "x\ny\ta\tb\n");
    }
}
