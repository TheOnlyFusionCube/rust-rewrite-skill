//! m01: parse basic sections/keys
use ini_mini::parse;

#[test]
fn m01_has_section() {
    let cfg = parse("[db]\nhost=localhost\n").unwrap();
    assert!(cfg.has_section("db"));
}

#[test]
fn m01_get_str_host() {
    let cfg = parse("[db]\nhost = localhost\nport = 5432\n").unwrap();
    assert_eq!(cfg.get_str("db", "host").unwrap(), "localhost");
}

#[test]
fn m01_get_str_port() {
    let cfg = parse("[db]\nhost = localhost\nport = 5432\n").unwrap();
    assert_eq!(cfg.get_str("db", "port").unwrap(), "5432");
}

#[test]
fn m01_multiple_sections() {
    let cfg = parse("[a]\nx=1\n[b]\ny=2\n").unwrap();
    assert!(cfg.has_section("a"));
    assert!(cfg.has_section("b"));
    assert_eq!(cfg.get_str("a", "x").unwrap(), "1");
    assert_eq!(cfg.get_str("b", "y").unwrap(), "2");
}

#[test]
fn m01_sections_order() {
    let cfg = parse("[z]\na=1\n[a]\nb=2\n[m]\nc=3\n").unwrap();
    assert_eq!(cfg.sections(), vec!["z", "a", "m"]);
}

#[test]
fn m01_keys_order() {
    let cfg = parse("[s]\nc=1\na=2\nb=3\n").unwrap();
    assert_eq!(cfg.keys("s").unwrap(), vec!["c", "a", "b"]);
}

#[test]
fn m01_has_key() {
    let cfg = parse("[s]\nk=v\n").unwrap();
    assert!(cfg.has_key("s", "k"));
    assert!(!cfg.has_key("s", "nope"));
    assert!(!cfg.has_key("missing", "k"));
}

#[test]
fn m01_empty_value() {
    let cfg = parse("[s]\nempty=\n").unwrap();
    assert_eq!(cfg.get_str("s", "empty").unwrap(), "");
}
