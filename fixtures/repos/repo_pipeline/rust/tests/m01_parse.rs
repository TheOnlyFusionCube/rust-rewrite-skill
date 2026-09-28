//! Milestone m01: parse_line.
use pipeline::parse_line;

#[test]
fn m01_ok() {
    assert_eq!(parse_line("a,3"), Some(("a".into(), 3)));
}

#[test]
fn m01_comment() {
    assert_eq!(parse_line("# comment"), None);
}

#[test]
fn m01_blank() {
    assert_eq!(parse_line(""), None);
}

#[test]
fn m01_malformed() {
    assert_eq!(parse_line("nope"), None);
}

#[test]
fn m01_bad_int() {
    assert_eq!(parse_line("x,abc"), None);
}
