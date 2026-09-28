//! m05: duplicate keys / sections — events + last-wins get
use inih_real::{parse_string, Ini};

#[test]
fn m05_three_events_same_key() {
    let o = parse_string("[s]\nk=first\nk=second\nk=third\n");
    assert_eq!(o.events.len(), 3);
    assert_eq!(o.events[0].value, "first");
    assert_eq!(o.events[1].value, "second");
    assert_eq!(o.events[2].value, "third");
}

#[test]
fn m05_get_last_wins() {
    let ini = Ini::parse("[s]\nk=first\nk=second\nk=third\n");
    assert_eq!(ini.get("s", "k"), Some("third"));
}

#[test]
fn m05_duplicate_section_header_merges_name() {
    let o = parse_string("[s]\nk=first\n[s]\nm=merged\n");
    assert_eq!(o.events.len(), 2);
    assert_eq!(o.events[0].section, "s");
    assert_eq!(o.events[1].section, "s");
    assert_eq!(o.events[1].name, "m");
    assert_eq!(o.events[1].value, "merged");
}

#[test]
fn m05_sections_lists_once() {
    let ini = Ini::parse("[s]\nk=1\n[s]\nm=2\n");
    assert_eq!(ini.sections(), vec!["s"]);
}

#[test]
fn m05_mixed_dup_events() {
    let o = parse_string("[main]\ndup=a\ndup=b\n");
    assert_eq!(o.events.len(), 2);
    let ini = Ini::parse("[main]\ndup=a\ndup=b\n");
    assert_eq!(ini.get("main", "dup"), Some("b"));
}
