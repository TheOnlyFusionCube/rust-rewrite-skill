//! m02: start-of-line and inline comments
use inih_real::parse_string;

#[test]
fn m02_hash_and_semicolon_line_comments() {
    let o = parse_string("; line\n# hash\n[s]\na=1\n; ignored\nb=2\n");
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events.len(), 2);
    assert_eq!(o.events[0].value, "1");
    assert_eq!(o.events[1].value, "2");
}

#[test]
fn m02_inline_semicolon_strips() {
    let o = parse_string("[s]\nb=2 ; inline comment\ne=5 ;Trailing\n");
    assert_eq!(o.events[0].value, "2");
    assert_eq!(o.events[1].value, "5");
}

#[test]
fn m02_hash_not_inline() {
    let o = parse_string("[s]\nc=3#not-inline\n");
    assert_eq!(o.events[0].value, "3#not-inline");
}

#[test]
fn m02_semicolon_no_ws_not_inline() {
    let o = parse_string("[s]\nd=4;also-not-inline-no-ws\n");
    assert_eq!(o.events[0].value, "4;also-not-inline-no-ws");
}

#[test]
fn m02_inline_edge_test1() {
    let o = parse_string("[comment_test]\ntest1 = 1;2;3 ; only this will be a comment\n");
    assert_eq!(o.events[0].value, "1;2;3");
}

#[test]
fn m02_inline_edge_test2_needs_ws() {
    let o = parse_string(
        "[comment_test]\ntest2 = 2;3;4;this won't be a comment, needs whitespace before ';'\n",
    );
    assert_eq!(
        o.events[0].value,
        "2;3;4;this won't be a comment, needs whitespace before ';'"
    );
}

#[test]
fn m02_key_with_semicolon() {
    let o = parse_string("[comment_test]\ntest;3 = 345 ; key should be test;3\n");
    assert_eq!(o.events[0].name, "test;3");
    assert_eq!(o.events[0].value, "345");
}

#[test]
fn m02_test7_blank_via_inline() {
    let o = parse_string("[comment_test]\ntest7 = ; blank value via inline\n");
    assert_eq!(o.events[0].value, "");
}

#[test]
fn m02_test8_no_ws_keeps_semicolon() {
    let o = parse_string(
        "[comment_test]\ntest8 =; not a comment, needs whitespace before ';'\n",
    );
    assert_eq!(
        o.events[0].value,
        "; not a comment, needs whitespace before ';'"
    );
}
