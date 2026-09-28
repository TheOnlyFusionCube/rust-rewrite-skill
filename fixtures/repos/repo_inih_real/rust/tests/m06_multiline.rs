//! m06: multiline continuations = separate events
use inih_real::{parse_string, Ini};

#[test]
fn m06_two_line_multi() {
    let o = parse_string("[section1]\nmulti = this is a\n        multi-line value\n");
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events.len(), 2);
    assert_eq!(o.events[0].name, "multi");
    assert_eq!(o.events[0].value, "this is a");
    assert_eq!(o.events[1].name, "multi");
    assert_eq!(o.events[1].value, "multi-line value");
}

#[test]
fn m06_surrounding_singles() {
    let o = parse_string(
        "[section1]\nsingle1 = abc\nmulti = this is a\n        multi-line value\nsingle2 = xyz\n",
    );
    assert_eq!(o.events.len(), 4);
    assert_eq!(o.events[0].value, "abc");
    assert_eq!(o.events[3].value, "xyz");
}

#[test]
fn m06_three_line_multi() {
    let o = parse_string("[section2]\nmulti = a\n        b\n        c\n");
    assert_eq!(o.events.len(), 3);
    assert_eq!(o.events[0].value, "a");
    assert_eq!(o.events[1].value, "b");
    assert_eq!(o.events[2].value, "c");
}

#[test]
fn m06_get_is_last_continuation() {
    let ini = Ini::parse("[section2]\nmulti = a\n        b\n        c\n");
    assert_eq!(ini.get("section2", "multi"), Some("c"));
}

#[test]
fn m06_section_resets_prev_name() {
    let o = parse_string("[a]\nmulti = x\n        y\n[b]\nz=1\n");
    assert_eq!(o.events.len(), 3);
    assert_eq!(o.events[2].section, "b");
    assert_eq!(o.events[2].name, "z");
}

#[test]
fn m06_mixed_fixture_cont() {
    let o = parse_string("# header\npre=1\n[main]\ncont = line1\n       line2\n");
    assert_eq!(o.events[0].section, "");
    assert_eq!(o.events[1].name, "cont");
    assert_eq!(o.events[1].value, "line1");
    assert_eq!(o.events[2].name, "cont");
    assert_eq!(o.events[2].value, "line2");
}
