//! m03: duplicates / ordering quirks
use ini_mini::parse;

#[test]
fn m03_duplicate_key_last_wins() {
    let cfg = parse("[s]\na=1\na=2\na=3\n").unwrap();
    assert_eq!(cfg.get_str("s", "a").unwrap(), "3");
}

#[test]
fn m03_duplicate_key_keeps_first_order() {
    let cfg = parse("[s]\na=1\nb=2\na=9\n").unwrap();
    assert_eq!(cfg.keys("s").unwrap(), vec!["a", "b"]);
    assert_eq!(cfg.get_str("s", "a").unwrap(), "9");
}

#[test]
fn m03_duplicate_section_merges() {
    let cfg = parse("[s]\na=1\n[s]\nb=2\n").unwrap();
    assert_eq!(cfg.sections(), vec!["s"]);
    assert_eq!(cfg.get_str("s", "a").unwrap(), "1");
    assert_eq!(cfg.get_str("s", "b").unwrap(), "2");
}

#[test]
fn m03_duplicate_section_last_wins_same_key() {
    let cfg = parse("[s]\na=1\n[s]\na=2\n").unwrap();
    assert_eq!(cfg.get_str("s", "a").unwrap(), "2");
}

#[test]
fn m03_keys_before_section_use_default() {
    let cfg = parse("pre=1\n[s]\nk=v\n").unwrap();
    assert!(cfg.has_section("DEFAULT"));
    assert_eq!(cfg.get_str("DEFAULT", "pre").unwrap(), "1");
    assert_eq!(cfg.sections()[0], "DEFAULT");
}

#[test]
fn m03_only_default_keys() {
    let cfg = parse("a=1\nb=2\n").unwrap();
    assert_eq!(cfg.sections(), vec!["DEFAULT"]);
    assert_eq!(cfg.get_str("DEFAULT", "b").unwrap(), "2");
}

#[test]
fn m03_case_sensitive_sections() {
    let cfg = parse("[A]\nx=1\n[a]\ny=2\n").unwrap();
    assert_eq!(cfg.sections(), vec!["A", "a"]);
    assert_eq!(cfg.get_str("A", "x").unwrap(), "1");
    assert_eq!(cfg.get_str("a", "y").unwrap(), "2");
}

#[test]
fn m03_case_sensitive_keys() {
    let cfg = parse("[s]\nKey=1\nkey=2\n").unwrap();
    assert_eq!(cfg.get_str("s", "Key").unwrap(), "1");
    assert_eq!(cfg.get_str("s", "key").unwrap(), "2");
}
