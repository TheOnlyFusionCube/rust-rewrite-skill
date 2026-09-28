//! Milestone m03: byte counting.
use wcapp::count_bytes;

#[test]
fn m03_empty() {
    assert_eq!(count_bytes(b""), 0);
}

#[test]
fn m03_ascii() {
    assert_eq!(count_bytes(b"hello world\n"), 12);
}

#[test]
fn m03_with_spaces() {
    assert_eq!(count_bytes(b"a b c"), 5);
}

#[test]
fn m03_multiline_len() {
    let text = "one\ntwo three\nfour\n";
    assert_eq!(count_bytes(text.as_bytes()), 19);
}
