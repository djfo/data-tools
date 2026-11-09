# data

Command-line tools for working with Excel documents.

## Binaries

- `xlsx` — convert the first worksheet of an `.xlsx` file to TSV on stdout.
  Supports selecting and reordering columns (`-f A,C,E`) and escaping
  special characters so each record stays on one line (`-e`).
- `xls_column_names` — print a TSV mapping from zero-based column indices
  to Excel column names (`0 -> A`, `26 -> AA`), optionally limited to N lines.
- `tsv-graph` — read a TSV file and emit a directed graph in DOT format,
  drawing one edge from each row's ID column to its target column. The
  optional `-n` flag attaches node labels from an additional column.

## Usage

```sh
cargo run --bin xlsx -- file.xlsx -f A,C,E -e
cargo run --bin xls_column_names -- 30
cargo run --bin tsv-graph -- file.tsv ID TargetID -n Name
```
