//! m04: keys before any section → section ""
use inih_real::{parse_string, Ini};

#[test]
fn m04_presection_empty_section_string() {
    let o = parse_string("alpha=before\nbeta = also\n[real]\ngamma=after\n");
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events[0].section, "");
    assert_eq!(o.events[0].name, "alpha");
    assert_eq!(o.events[0].value, "before");
    assert_eq!(o.events[1].section, "");
    assert_eq!(o.events[1].name, "beta");
    assert_eq!(o.events[2].section, "real");
    assert_eq!(o.events[2].name, "gamma");
}

#[test]
fn m04_get_empty_section() {
    let ini = Ini::parse("alpha=before\n[real]\ngamma=after\n");
    assert_eq!(ini.get("", "alpha"), Some("before"));
    assert_eq!(ini.get("real", "gamma"), Some("after"));
}

#[test]
fn m04_sections_includes_empty() {
    let ini = Ini::parse("pre=1\n[main]\nx=2\n");
    assert_eq!(ini.sections(), vec!["", "main"]);
}

#[test]
fn m04_empty_bracket_section() {
    let o = parse_string("[]\nx=1\n[a]\ny=2\n");
    assert_eq!(o.events[0].section, "");
    assert_eq!(o.events[0].name, "x");
    assert_eq!(o.events[1].section, "a");
}

#[test]
fn m04_not_default_literal() {
    let ini = Ini::parse("k=v\n");
    assert!(ini.has_section(""));
    assert!(!ini.has_section("DEFAULT"));
    assert_eq!(ini.get("DEFAULT", "k"), None);
    assert_eq!(ini.get("", "k"), Some("v"));
}
