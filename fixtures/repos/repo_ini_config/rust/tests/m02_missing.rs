//! Milestone m02: missing keys/sections + defaults.
use inilib::parse;

const SAMPLE: &str = r#"
[database]
host = localhost
"#;

#[test]
fn m02_missing_section_key() {
    let cfg = parse(SAMPLE);
    assert_eq!(cfg.get("missing", "x", None), None);
}

#[test]
fn m02_missing_key_with_default() {
    let cfg = parse(SAMPLE);
    assert_eq!(
        cfg.get("database", "nope", Some("fallback")).as_deref(),
        Some("fallback")
    );
}

#[test]
fn m02_missing_section_bool() {
    let cfg = parse(SAMPLE);
    assert!(!cfg.has_section("nope"));
}

#[test]
fn m02_present_key_ignores_default() {
    let cfg = parse(SAMPLE);
    assert_eq!(
        cfg.get("database", "host", Some("other")).as_deref(),
        Some("localhost")
    );
}
