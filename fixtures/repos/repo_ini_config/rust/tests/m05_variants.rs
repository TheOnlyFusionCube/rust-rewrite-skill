//! Milestone m05: bool variants + bad int via Config::set.
use inilib::Config;

#[test]
fn m05_bool_true_variants() {
    let mut c = Config::new();
    for v in ["true", "YES", "1", "On"] {
        c.set("s", "b", v);
        assert!(c.get_bool("s", "b", false), "expected true for {v}");
    }
}

#[test]
fn m05_bool_false_variants() {
    let mut c = Config::new();
    for v in ["false", "NO", "0", "off"] {
        c.set("s", "b", v);
        assert!(!c.get_bool("s", "b", true), "expected false for {v}");
    }
}

#[test]
fn m05_int_bad_falls_back() {
    let mut c = Config::new();
    c.set("s", "n", "abc");
    assert_eq!(c.get_int("s", "n", 7), 7);
}

#[test]
fn m05_ensure_section_visible() {
    let mut c = Config::new();
    c.ensure_section("new");
    assert!(c.has_section("new"));
}
