//! Milestone m01: line counting (classic wc rules).
use wcapp::count_lines;

#[test]
fn m01_empty() {
    assert_eq!(count_lines(""), 0);
}

#[test]
fn m01_single_newline() {
    assert_eq!(count_lines("hello world\n"), 1);
}

#[test]
fn m01_no_trailing_newline() {
    assert_eq!(count_lines("a b c"), 1);
}

#[test]
fn m01_multiline() {
    assert_eq!(count_lines("one\ntwo three\nfour\n"), 3);
}

#[test]
fn m01_only_newlines() {
    assert_eq!(count_lines("\n\n"), 2);
}
