# data

Command-line tools for working with Excel documents and TSV tables.

## Binaries

- `xlsx` — convert the first worksheet of an `.xlsx` file to TSV on stdout.
  Supports selecting and reordering columns, including inclusive ranges
  (`-f A,C,E` or `-f A:D,P,Q,AA:AZ`) and escaping special characters so
  each record stays on one line (`-e`).
- `xls_column_names` — print a TSV mapping from zero-based column indices
  to Excel column names (`0 -> A`, `26 -> AA`), optionally limited to N lines.
- `tsv-to-md` — convert a TSV file to a Markdown table on stdout, with
  per-column alignment via `-a` (e.g. `-a llrc` for left, left, right,
  center; `l` = left, `r` = right, `c` = center, `-` = default).

## Usage

```sh
cargo run --bin xlsx -- file.xlsx -f A:D,P,Q,AA:AZ -e
cargo run --bin xls_column_names -- 30
cargo run --bin tsv-to-md -- table.tsv -a llrc
```
