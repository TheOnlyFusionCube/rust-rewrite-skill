//! m07: unicode / escapes
use ini_mini::parse;

#[test]
fn m07_unicode_value() {
    let cfg = parse("[u]\nname = café\n").unwrap();
    assert_eq!(cfg.get_str("u", "name").unwrap(), "café");
}

#[test]
fn m07_unicode_section_and_key() {
    let cfg = parse("[セクション]\nキー = 値\n").unwrap();
    assert_eq!(cfg.get_str("セクション", "キー").unwrap(), "値");
}

#[test]
fn m07_escape_newline() {
    let cfg = parse("[u]\npath = a\\nb\n").unwrap();
    assert_eq!(cfg.get_str("u", "path").unwrap(), "a\nb");
}

#[test]
fn m07_escape_tab() {
    let cfg = parse("[u]\npath = a\\tb\n").unwrap();
    assert_eq!(cfg.get_str("u", "path").unwrap(), "a\tb");
}

#[test]
fn m07_escape_backslash() {
    let cfg = parse("[u]\npath = a\\\\b\n").unwrap();
    assert_eq!(cfg.get_str("u", "path").unwrap(), "a\\b");
}

#[test]
fn m07_escape_hash_semicolon() {
    let cfg = parse("[u]\npath = a\\#b\\;c\n").unwrap();
    assert_eq!(cfg.get_str("u", "path").unwrap(), "a#b;c");
}

#[test]
fn m07_unknown_escape_keeps_char() {
    let cfg = parse("[u]\npath = a\\qb\n").unwrap();
    assert_eq!(cfg.get_str("u", "path").unwrap(), "aqb");
}

#[test]
fn m07_escape_roundtrip() {
    let cfg = parse("[u]\npath = a\\nb\\tc\\\\d\\#x\\;y\n").unwrap();
    let expected = "a\nb\tc\\d#x;y";
    assert_eq!(cfg.get_str("u", "path").unwrap(), expected);
    let cfg2 = parse(&cfg.to_string()).unwrap();
    assert_eq!(cfg2.get_str("u", "path").unwrap(), expected);
}
