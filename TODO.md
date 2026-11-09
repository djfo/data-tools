# TODO

## Deferred Work

- Make `xlsx` accept all spreadsheet formats calamine supports (`.xlsx`,
  `.xlsb`, `.xls`, `.ods`) instead of only `.xlsx`: switch `read_first_sheet`
  in `src/lib.rs` from `open_workbook` with the pinned `calamine::Xlsx` reader
  to `open_workbook_auto` and match on the returned `Sheets` enum.

## Tools

- `tsv-graph`: read a TSV file with ID and target columns and emit a
  directed graph in DOT format (one edge per row, pointing to the target),
  with optional node labels. Implemented as the `tsv-graph` binary.
- `tsv-transpose`: transpose a TSV read from stdin to stdout (rows become
  columns), a dependency-free alternative to `datamash transpose`. Decide
  padding for ragged input; proposal: pad short rows with empty fields, since
  `xlsx` emits a rectangular grid anyway.
