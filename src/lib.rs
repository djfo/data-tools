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

/// Parse a comma-separated list of Excel column references into zero-based indices.
pub fn parse_columns(spec: &str) -> Result<Vec<usize>, Error> {
    spec.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| column_index(s).ok_or_else(|| Error::InvalidColumn(s.to_string())))
        .collect()
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

/// Render a grid as TSV, keeping only `columns` (zero-based) when given.
pub fn to_tsv(rows: &[Vec<String>], columns: Option<&[usize]>) -> String {
    let mut out = String::new();
    for row in rows {
        let mut first = true;
        match columns {
            None => {
                for cell in row {
                    if !first {
                        out.push('\t');
                    }
                    first = false;
                    out.push_str(cell);
                }
            }
            Some(cols) => {
                for &i in cols {
                    if !first {
                        out.push('\t');
                    }
                    first = false;
                    if let Some(cell) = row.get(i) {
                        out.push_str(cell);
                    }
                }
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
    fn parse_columns_splits_and_trims() {
        assert_eq!(parse_columns("A, C ,e").unwrap(), vec![0, 2, 4]);
        assert!(parse_columns("A,,B").is_ok());
        assert!(parse_columns("A,x1").is_err());
    }

    #[test]
    fn to_tsv_without_selection() {
        let rows = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["1".to_string(), "2".to_string()],
        ];
        assert_eq!(to_tsv(&rows, None), "a\tb\n1\t2\n");
    }

    #[test]
    fn to_tsv_with_selection_reorders() {
        let rows = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
        ];
        assert_eq!(to_tsv(&rows, Some(&[2, 0])), "c\ta\n3\t1\n");
    }

    #[test]
    fn to_tsv_missing_column_is_empty() {
        let rows = vec![vec!["a".to_string()]];
        assert_eq!(to_tsv(&rows, Some(&[0, 5])), "a\t\n");
    }
}
