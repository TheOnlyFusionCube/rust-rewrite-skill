//! Milestone m04: get_bool / get_int from parsed sample.
use inilib::parse;

const SAMPLE: &str = r#"
[database]
host = localhost
port = 5432
enabled = yes

[feature]
flag = true
count = 3
"#;

#[test]
fn m04_bool_enabled() {
    let cfg = parse(SAMPLE);
    assert!(cfg.get_bool("database", "enabled", false));
}

#[test]
fn m04_bool_flag() {
    let cfg = parse(SAMPLE);
    assert!(cfg.get_bool("feature", "flag", false));
}

#[test]
fn m04_bool_absent_default() {
    let cfg = parse(SAMPLE);
    assert!(!cfg.get_bool("database", "absent", false));
}

#[test]
fn m04_int_port() {
    let cfg = parse(SAMPLE);
    assert_eq!(cfg.get_int("database", "port", 0), 5432);
}

#[test]
fn m04_int_count_and_default() {
    let cfg = parse(SAMPLE);
    assert_eq!(cfg.get_int("feature", "count", 0), 3);
    assert_eq!(cfg.get_int("feature", "absent", 9), 9);
}
