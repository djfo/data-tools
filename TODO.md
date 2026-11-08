# TODO

## Deferred Work

- Make `xlsx2tsv` accept all spreadsheet formats calamine supports (`.xlsx`,
  `.xlsb`, `.xls`, `.ods`) instead of only `.xlsx`: switch `read_first_sheet`
  in `src/lib.rs` from `open_workbook` with the pinned `calamine::Xlsx` reader
  to `open_workbook_auto` and match on the returned `Sheets` enum.
