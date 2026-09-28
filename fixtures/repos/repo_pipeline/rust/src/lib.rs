//! Port of Python pipeline: parse CSV-like lines → filter → aggregate → write.
//! Implement so milestone tests under tests/ pass.
//! Do not weaken, delete, or edit files under tests/. Prefer std-only.

/// Parse one line into (key, value). Skip blanks, `#` comments, malformed lines.
pub fn parse_line(line: &str) -> Option<(String, i64)> {
    todo!("parse_line")
}

/// Keep rows where value > 0.
pub fn filter_rows(rows: &[(String, i64)]) -> Vec<(String, i64)> {
    todo!("filter_rows")
}

/// Sum values by key; return sorted by key ascending.
pub fn aggregate(rows: &[(String, i64)]) -> Vec<(String, i64)> {
    todo!("aggregate")
}

/// Full transform over raw lines.
pub fn run_pipeline(lines: &[String]) -> Vec<(String, i64)> {
    todo!("run_pipeline")
}

/// Read file into lines (split on `\n`; trailing empty from final newline ok).
pub fn read_rows(path: &str) -> std::io::Result<Vec<String>> {
    todo!("read_rows")
}

/// Write `key,sum` lines (sorted already) with trailing newline if non-empty.
pub fn write_aggregates(path: &str, items: &[(String, i64)]) -> std::io::Result<()> {
    todo!("write_aggregates")
}
