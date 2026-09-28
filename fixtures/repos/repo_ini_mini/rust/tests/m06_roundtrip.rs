//! m06: round-trip / print
use ini_mini::{parse, Config};

#[test]
fn m06_to_string_basic() {
    let mut cfg = Config::new();
    cfg.set("db", "host", "localhost");
    cfg.set("db", "port", "5432");
    let s = cfg.to_string();
    assert!(s.contains("[db]"));
    assert!(s.contains("host = localhost"));
    assert!(s.contains("port = 5432"));
}

#[test]
fn m06_roundtrip_parse_print() {
    let original = "[db]\nhost = localhost\nport = 5432\n\n[app]\nname = demo\n";
    let cfg = parse(original).unwrap();
    let text = cfg.to_string();
    let cfg2 = parse(&text).unwrap();
    assert_eq!(cfg2.sections(), vec!["db", "app"]);
    assert_eq!(cfg2.get_str("db", "host").unwrap(), "localhost");
    assert_eq!(cfg2.get_str("app", "name").unwrap(), "demo");
}

#[test]
fn m06_blank_line_between_sections() {
    let mut cfg = Config::new();
    cfg.set("a", "x", "1");
    cfg.set("b", "y", "2");
    let s = cfg.to_string();
    assert!(s.contains("]\n\n[") || s.contains("x = 1\n\n[b]"), "got: {s:?}");
}

#[test]
fn m06_spaces_around_equals_in_stringify() {
    let mut cfg = Config::new();
    cfg.set("s", "k", "v");
    let s = cfg.to_string();
    assert!(s.contains("k = v"), "got: {s:?}");
}

#[test]
fn m06_trailing_newline() {
    let mut cfg = Config::new();
    cfg.set("s", "k", "v");
    let s = cfg.to_string();
    assert!(s.ends_with('\n'));
}

#[test]
fn m06_empty_config_string() {
    let cfg = Config::new();
    assert_eq!(cfg.to_string(), "");
}

#[test]
fn m06_roundtrip_preserves_empty_value() {
    let cfg = parse("[s]\nempty=\n").unwrap();
    let cfg2 = parse(&cfg.to_string()).unwrap();
    assert_eq!(cfg2.get_str("s", "empty").unwrap(), "");
}

#[test]
fn m06_manual_set_then_get() {
    let mut cfg = Config::new();
    cfg.ensure_section("s");
    cfg.set("s", "k", "v");
    assert!(cfg.has_section("s"));
    assert_eq!(cfg.get_str("s", "k").unwrap(), "v");
}
