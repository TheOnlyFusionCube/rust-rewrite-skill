//! m08: BOM, section spaces, mixed adversarial
use inih_real::{parse_string, Ini};

#[test]
fn m08_utf8_bom_stripped() {
    let text = "\u{FEFF}[bom_section]\nbom_name=bom_value\n";
    let o = parse_string(text);
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events.len(), 1);
    assert_eq!(o.events[0].section, "bom_section");
    assert_eq!(o.events[0].name, "bom_name");
    assert_eq!(o.events[0].value, "bom_value");
}

#[test]
fn m08_section_name_preserves_inner_spaces() {
    let o = parse_string("[ section 2 ]\nhappy=4\n");
    assert_eq!(o.events[0].section, " section 2 ");
    let ini = Ini::parse("[ section 2 ]\nhappy=4\n");
    assert_eq!(ini.get(" section 2 ", "happy"), Some("4"));
    assert_eq!(ini.get("section 2", "happy"), None);
}

#[test]
fn m08_mixed_full_stream() {
    let text = "# header\npre=1\n[main]\nname = value ; c\ncont = line1\n       line2\ndup=a\ndup=b\n[other]\nz: 99\n";
    let o = parse_string(text);
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events.len(), 7);
    assert_eq!(o.events[0].section, "");
    assert_eq!(o.events[0].name, "pre");
    assert_eq!(o.events[1].name, "name");
    assert_eq!(o.events[1].value, "value");
    assert_eq!(o.events[2].value, "line1");
    assert_eq!(o.events[3].value, "line2");
    assert_eq!(o.events[4].value, "a");
    assert_eq!(o.events[5].value, "b");
    assert_eq!(o.events[6].section, "other");
    assert_eq!(o.events[6].name, "z");
    assert_eq!(o.events[6].value, "99");
}

#[test]
fn m08_mixed_get_last_wins() {
    let ini = Ini::parse("# header\npre=1\n[main]\ndup=a\ndup=b\n[other]\nz: 99\n");
    assert_eq!(ini.get("", "pre"), Some("1"));
    assert_eq!(ini.get("main", "dup"), Some("b"));
    assert_eq!(ini.get("other", "z"), Some("99"));
}

#[test]
fn m08_blank_lines_ignored() {
    let o = parse_string("\n\n[s]\n\na=1\n\n\nb=2\n");
    assert_eq!(o.events.len(), 2);
}

#[test]
fn m08_case_sensitive() {
    let ini = Ini::parse("[DB]\nHost=x\n");
    assert!(ini.has_section("DB"));
    assert!(!ini.has_section("db"));
    assert_eq!(ini.get("DB", "Host"), Some("x"));
    assert_eq!(ini.get("DB", "host"), None);
}

#[test]
fn m08_only_comments_ok() {
    let o = parse_string("; a\n# b\n");
    assert_eq!(o.error_line, 0);
    assert!(o.events.is_empty());
}
