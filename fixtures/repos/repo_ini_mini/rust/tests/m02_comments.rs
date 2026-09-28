//! m02: comments / whitespace
use ini_mini::parse;

#[test]
fn m02_hash_comment_ignored() {
    let cfg = parse("# top\n[s]\nk=v\n").unwrap();
    assert_eq!(cfg.get_str("s", "k").unwrap(), "v");
}

#[test]
fn m02_semicolon_comment_ignored() {
    let cfg = parse("; mid\n[s]\nk=v\n").unwrap();
    assert_eq!(cfg.get_str("s", "k").unwrap(), "v");
}

#[test]
fn m02_comment_after_indent() {
    let cfg = parse("   # indented comment\n[s]\nk=v\n").unwrap();
    assert!(cfg.has_section("s"));
}

#[test]
fn m02_blank_lines() {
    let cfg = parse("\n\n[s]\n\nk=v\n\n").unwrap();
    assert_eq!(cfg.get_str("s", "k").unwrap(), "v");
}

#[test]
fn m02_trim_key_value() {
    let cfg = parse("[s]\n  key  =  value  \n").unwrap();
    assert_eq!(cfg.get_str("s", "key").unwrap(), "value");
}

#[test]
fn m02_trim_section_name() {
    let cfg = parse("[  spaced  ]\nk=v\n").unwrap();
    assert!(cfg.has_section("spaced"));
    assert_eq!(cfg.get_str("spaced", "k").unwrap(), "v");
}

#[test]
fn m02_internal_spaces_in_value_kept() {
    let cfg = parse("[s]\nmsg = hello  world\n").unwrap();
    assert_eq!(cfg.get_str("s", "msg").unwrap(), "hello  world");
}

#[test]
fn m02_no_inline_comment() {
    let cfg = parse("[s]\nurl = http://x#frag\n").unwrap();
    assert_eq!(cfg.get_str("s", "url").unwrap(), "http://x#frag");
}
