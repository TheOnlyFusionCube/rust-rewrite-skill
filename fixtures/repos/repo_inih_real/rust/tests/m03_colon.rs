//! m03: name:value form
use inih_real::{parse_string, Ini};

#[test]
fn m03_content_type_colon() {
    let o = parse_string("[colon_tests]\nContent-Type: text/html\n");
    assert_eq!(o.events[0].name, "Content-Type");
    assert_eq!(o.events[0].value, "text/html");
}

#[test]
fn m03_foo_colon_bar() {
    let o = parse_string("[colon_tests]\nfoo:bar\n");
    assert_eq!(o.events[0].name, "foo");
    assert_eq!(o.events[0].value, "bar");
}

#[test]
fn m03_adams_spaced() {
    let o = parse_string("[colon_tests]\nadams : 42\n");
    assert_eq!(o.events[0].value, "42");
}

#[test]
fn m03_equals_inside_colon_value() {
    let o = parse_string("[colon_tests]\nfunny1 : with = equals\n");
    assert_eq!(o.events[0].value, "with = equals");
}

#[test]
fn m03_colon_inside_equals_value() {
    let o = parse_string("[colon_tests]\nfunny2 = with : colons\n");
    assert_eq!(o.events[0].value, "with : colons");
}

#[test]
fn m03_get_via_ini() {
    let ini = Ini::parse("[colon_tests]\nContent-Type: text/html\nfoo:bar\n");
    assert_eq!(ini.get("colon_tests", "Content-Type"), Some("text/html"));
    assert_eq!(ini.get("colon_tests", "foo"), Some("bar"));
}
