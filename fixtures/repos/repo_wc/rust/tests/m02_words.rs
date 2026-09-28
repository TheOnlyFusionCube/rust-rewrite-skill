//! Milestone m02: word counting (split_whitespace).
use wcapp::count_words;

#[test]
fn m02_empty() {
    assert_eq!(count_words(""), 0);
}

#[test]
fn m02_simple() {
    assert_eq!(count_words("hello world\n"), 2);
}

#[test]
fn m02_no_trailing_newline() {
    assert_eq!(count_words("a b c"), 3);
}

#[test]
fn m02_extra_whitespace() {
    assert_eq!(count_words("  one   two\tthree  \n"), 3);
}

#[test]
fn m02_multiline() {
    assert_eq!(count_words("one\ntwo three\nfour\n"), 4);
}
