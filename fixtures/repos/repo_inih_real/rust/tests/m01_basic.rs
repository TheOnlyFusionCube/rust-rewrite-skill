//! m01: basic sections, keys, whitespace trim
use inih_real::{parse_string, Ini};

#[test]
fn m01_two_keys() {
    let o = parse_string("[section1]\none=This is a test\ntwo = 1234\n");
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events.len(), 2);
    assert_eq!(o.events[0].section, "section1");
    assert_eq!(o.events[0].name, "one");
    assert_eq!(o.events[0].value, "This is a test");
    assert_eq!(o.events[1].name, "two");
    assert_eq!(o.events[1].value, "1234");
}

#[test]
fn m01_whitespace_trim() {
    let o = parse_string("[db]\n  host   =   localhost  \nport=5432\n");
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events[0].name, "host");
    assert_eq!(o.events[0].value, "localhost");
    assert_eq!(o.events[1].name, "port");
    assert_eq!(o.events[1].value, "5432");
}

#[test]
fn m01_get_last_wins_single() {
    let ini = Ini::parse("[db]\nhost=localhost\nport=5432\n");
    assert_eq!(ini.get("db", "host"), Some("localhost"));
    assert_eq!(ini.get("db", "port"), Some("5432"));
    assert_eq!(ini.get("db", "missing"), None);
}

#[test]
fn m01_has_section() {
    let ini = Ini::parse("[a]\nx=1\n[b]\ny=2\n");
    assert!(ini.has_section("a"));
    assert!(ini.has_section("b"));
    assert!(!ini.has_section("c"));
}

#[test]
fn m01_sections_order() {
    let ini = Ini::parse("[z]\na=1\n[a]\nb=2\n[m]\nc=3\n");
    assert_eq!(ini.sections(), vec!["z", "a", "m"]);
}

#[test]
fn m01_empty_value() {
    let o = parse_string("[s]\nempty=\nblank = \nok=x\n");
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events[0].value, "");
    assert_eq!(o.events[1].value, "");
    assert_eq!(o.events[2].value, "x");
}
