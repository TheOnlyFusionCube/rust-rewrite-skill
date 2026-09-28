//! Milestone m01: sections and basic key get.
use inilib::parse;

const SAMPLE: &str = r#"
# top comment
[database]
host = localhost
port = 5432
; debug off
enabled = yes

[feature]
flag = true
count = 3
empty =
"#;

#[test]
fn m01_has_database() {
    let cfg = parse(SAMPLE);
    assert!(cfg.has_section("database"));
}

#[test]
fn m01_host() {
    let cfg = parse(SAMPLE);
    assert_eq!(cfg.get("database", "host", None).as_deref(), Some("localhost"));
}

#[test]
fn m01_port_string() {
    let cfg = parse(SAMPLE);
    assert_eq!(cfg.get("database", "port", None).as_deref(), Some("5432"));
}

#[test]
fn m01_feature_flag() {
    let cfg = parse(SAMPLE);
    assert_eq!(cfg.get("feature", "flag", None).as_deref(), Some("true"));
}

#[test]
fn m01_empty_value() {
    let cfg = parse(SAMPLE);
    assert_eq!(cfg.get("feature", "empty", None).as_deref(), Some(""));
}
