//! Milestone m03: comments / blank-only input + DEFAULT section.
use inilib::parse;

#[test]
fn m03_comments_only() {
    let cfg = parse("# hi\n; there\n");
    assert!(!cfg.has_section("database"));
}

#[test]
fn m03_blank_lines() {
    let cfg = parse("\n\n\n");
    assert!(!cfg.has_section("x"));
}

#[test]
fn m03_default_section_before_header() {
    let cfg = parse("name = early\n[sec]\nk = v\n");
    assert_eq!(cfg.get("DEFAULT", "name", None).as_deref(), Some("early"));
    assert_eq!(cfg.get("sec", "k", None).as_deref(), Some("v"));
}

#[test]
fn m03_inline_comment_lines_ignored() {
    let cfg = parse("[a]\n# ignored\nx = 1\n; also\n");
    assert_eq!(cfg.get("a", "x", None).as_deref(), Some("1"));
}
