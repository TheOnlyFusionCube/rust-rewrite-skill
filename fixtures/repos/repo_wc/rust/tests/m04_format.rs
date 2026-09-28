//! Milestone m04: count_text + format_counts.
use wcapp::{count_text, format_counts};

#[test]
fn m04_count_text_empty() {
    assert_eq!(count_text(""), (0, 0, 0));
}

#[test]
fn m04_count_text_simple() {
    assert_eq!(count_text("hello world\n"), (1, 2, 12));
}

#[test]
fn m04_count_text_multiline() {
    assert_eq!(count_text("one\ntwo three\nfour\n"), (3, 4, 19));
}

#[test]
fn m04_format_basic() {
    assert_eq!(format_counts(1, 2, 3, "x"), "1 2 3 x");
}

#[test]
fn m04_format_path() {
    assert_eq!(format_counts(2, 5, 24, "sample.txt"), "2 5 24 sample.txt");
}
