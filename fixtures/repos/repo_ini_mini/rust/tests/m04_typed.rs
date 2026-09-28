//! m04: typed getters + bools
use ini_mini::parse;

#[test]
fn m04_get_int_basic() {
    let cfg = parse("[t]\ni=42\n").unwrap();
    assert_eq!(cfg.get_int("t", "i").unwrap(), 42);
}

#[test]
fn m04_get_int_negative() {
    let cfg = parse("[t]\ni=-7\n").unwrap();
    assert_eq!(cfg.get_int("t", "i").unwrap(), -7);
}

#[test]
fn m04_get_int_plus() {
    let cfg = parse("[t]\ni=+3\n").unwrap();
    assert_eq!(cfg.get_int("t", "i").unwrap(), 3);
}

#[test]
fn m04_get_bool_true_variants() {
    for (k, v) in [("a", "true"), ("b", "YES"), ("c", "1"), ("d", "On")] {
        let text = format!("[t]\n{k}={v}\n");
        let cfg = parse(&text).unwrap();
        assert_eq!(cfg.get_bool("t", k).unwrap(), true, "{k}={v}");
    }
}

#[test]
fn m04_get_bool_false_variants() {
    for (k, v) in [("a", "false"), ("b", "NO"), ("c", "0"), ("d", "Off")] {
        let text = format!("[t]\n{k}={v}\n");
        let cfg = parse(&text).unwrap();
        assert_eq!(cfg.get_bool("t", k).unwrap(), false, "{k}={v}");
    }
}

#[test]
fn m04_get_str_or_default() {
    let cfg = parse("[t]\nk=v\n").unwrap();
    assert_eq!(cfg.get_str_or("t", "k", "d"), "v");
    assert_eq!(cfg.get_str_or("t", "missing", "d"), "d");
    assert_eq!(cfg.get_str_or("nope", "k", "d"), "d");
}

#[test]
fn m04_get_int_or_default() {
    let cfg = parse("[t]\ni=5\nbad=x\n").unwrap();
    assert_eq!(cfg.get_int_or("t", "i", 9), 5);
    assert_eq!(cfg.get_int_or("t", "bad", 9), 9);
    assert_eq!(cfg.get_int_or("t", "missing", 9), 9);
}

#[test]
fn m04_get_bool_or_default() {
    let cfg = parse("[t]\nb=yes\nbad=maybe\n").unwrap();
    assert_eq!(cfg.get_bool_or("t", "b", false), true);
    assert_eq!(cfg.get_bool_or("t", "bad", true), true);
    assert_eq!(cfg.get_bool_or("t", "missing", false), false);
}
