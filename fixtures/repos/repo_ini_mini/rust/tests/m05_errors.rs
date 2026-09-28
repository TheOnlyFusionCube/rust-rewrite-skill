//! m05: errors — exact message shapes
use ini_mini::{parse, IniError};

#[test]
fn m05_unclosed_section() {
    let err = parse("[open\nk=v\n").unwrap_err();
    assert_eq!(err.to_string(), "unclosed section on line 1");
}

#[test]
fn m05_unclosed_section_line2() {
    let err = parse("[ok]\n[bad\n").unwrap_err();
    assert_eq!(err.to_string(), "unclosed section on line 2");
}

#[test]
fn m05_section_garbage() {
    let err = parse("[a] junk\n").unwrap_err();
    assert_eq!(err.to_string(), "section header garbage on line 1");
}

#[test]
fn m05_empty_key() {
    let err = parse("[a]\n= v\n").unwrap_err();
    assert_eq!(err.to_string(), "empty key on line 2");
}

#[test]
fn m05_missing_section() {
    let cfg = parse("[a]\nk=v\n").unwrap();
    let err = cfg.get_str("nope", "k").unwrap_err();
    assert_eq!(err.to_string(), "section 'nope' not found");
    assert!(matches!(err, IniError::MissingSection { .. }));
}

#[test]
fn m05_missing_key() {
    let cfg = parse("[a]\nk=v\n").unwrap();
    let err = cfg.get_str("a", "nope").unwrap_err();
    assert_eq!(err.to_string(), "key 'nope' not found in section 'a'");
}

#[test]
fn m05_keys_missing_section() {
    let cfg = parse("[a]\nk=v\n").unwrap();
    let err = cfg.keys("nope").unwrap_err();
    assert_eq!(err.to_string(), "section 'nope' not found");
}

#[test]
fn m05_bad_int() {
    let cfg = parse("[t]\ni=1.5\n").unwrap();
    let err = cfg.get_int("t", "i").unwrap_err();
    assert_eq!(
        err.to_string(),
        "value '1.5' for 't.i' is not a valid integer"
    );
}

#[test]
fn m05_bad_int_empty() {
    let cfg = parse("[t]\ni=\n").unwrap();
    let err = cfg.get_int("t", "i").unwrap_err();
    assert_eq!(
        err.to_string(),
        "value '' for 't.i' is not a valid integer"
    );
}

#[test]
fn m05_bad_bool() {
    let cfg = parse("[t]\nb=maybe\n").unwrap();
    let err = cfg.get_bool("t", "b").unwrap_err();
    assert_eq!(
        err.to_string(),
        "value 'maybe' for 't.b' is not a valid boolean"
    );
}
