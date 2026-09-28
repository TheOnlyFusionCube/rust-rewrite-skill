//! m08: adversarial — huge lines, empty section, weird names, lenient lines
use ini_mini::parse;

#[test]
fn m08_empty_section_name() {
    let cfg = parse("[]\nx=1\n").unwrap();
    assert!(cfg.has_section(""));
    assert_eq!(cfg.get_str("", "x").unwrap(), "1");
}

#[test]
fn m08_weird_section_chars() {
    let cfg = parse("[a.b-c_d]\nk=v\n").unwrap();
    assert_eq!(cfg.get_str("a.b-c_d", "k").unwrap(), "v");
}

#[test]
fn m08_huge_key_and_value() {
    let key = "k".repeat(200);
    let val = "v".repeat(500);
    let text = format!("[h]\n{key} = {val}\n");
    let cfg = parse(&text).unwrap();
    assert_eq!(cfg.get_str("h", &key).unwrap(), val);
}

#[test]
fn m08_non_assignment_line_ignored() {
    let cfg = parse("[s]\nthis has no equals\nk=v\n").unwrap();
    assert_eq!(cfg.get_str("s", "k").unwrap(), "v");
}

#[test]
fn m08_equals_in_value() {
    let cfg = parse("[s]\nexpr = a=b=c\n").unwrap();
    assert_eq!(cfg.get_str("s", "expr").unwrap(), "a=b=c");
}

#[test]
fn m08_many_sections() {
    let mut text = String::new();
    for i in 0..30 {
        text.push_str(&format!("[s{i}]\nv={i}\n"));
    }
    let cfg = parse(&text).unwrap();
    assert_eq!(cfg.sections().len(), 30);
    assert_eq!(cfg.get_str("s29", "v").unwrap(), "29");
}

#[test]
fn m08_crlf_lines() {
    let cfg = parse("[s]\r\nk=v\r\n").unwrap();
    assert_eq!(cfg.get_str("s", "k").unwrap(), "v");
}

#[test]
fn m08_get_int_rejects_plus_only() {
    let cfg = parse("[t]\ni=+\n").unwrap();
    let err = cfg.get_int("t", "i").unwrap_err();
    assert!(err.to_string().contains("not a valid integer"));
}

#[test]
fn m08_bool_whitespace_inside() {
    let cfg = parse("[t]\nb=  yes  \n").unwrap();
    assert_eq!(cfg.get_bool("t", "b").unwrap(), true);
}

#[test]
fn m08_set_overwrites() {
    let mut cfg = parse("[s]\na=1\n").unwrap();
    cfg.set("s", "a", "2");
    assert_eq!(cfg.get_str("s", "a").unwrap(), "2");
}
