//! Port of Python wcapp: count lines, words, and bytes.
//! Implement these so the milestone tests under tests/ pass.
//! Do not weaken, delete, or edit files under tests/.

/// Count lines like classic `wc`: newlines; if text is non-empty and does not
/// end with `\n`, count the final partial line.
pub fn count_lines(text: &str) -> usize {
    todo!("implement count_lines")
}

/// Whitespace-separated word count (`str.split_whitespace` semantics).
pub fn count_words(text: &str) -> usize {
    todo!("implement count_words")
}

/// Byte length of `data`.
pub fn count_bytes(data: &[u8]) -> usize {
    todo!("implement count_bytes")
}

/// Return (lines, words, bytes) for UTF-8 text (bytes = text.as_bytes().len()).
pub fn count_text(text: &str) -> (usize, usize, usize) {
    todo!("implement count_text")
}

/// Read file as bytes, decode lossy UTF-8 for line/word counts, return
/// (lines, words, byte_len).
pub fn count_file(path: &str) -> std::io::Result<(usize, usize, usize)> {
    todo!("implement count_file")
}

/// Format: "{lines} {words} {bytes} {label}"
pub fn format_counts(lines: usize, words: usize, nbytes: usize, label: &str) -> String {
    todo!("implement format_counts")
}
