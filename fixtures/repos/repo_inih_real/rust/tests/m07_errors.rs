//! m07: parse errors — line numbers; keep parsing
use inih_real::{parse_string, Ini};

#[test]
fn m07_bad_section_error_line() {
    let o = parse_string("[section1]\nok=1\n[bad\nstill=parsed\n");
    assert_eq!(o.error_line, 3);
    assert_eq!(o.events.len(), 2);
    assert_eq!(o.events[0].value, "1");
    // section not updated on bad header → still in section1
    assert_eq!(o.events[1].section, "section1");
    assert_eq!(o.events[1].name, "still");
    assert_eq!(o.events[1].value, "parsed");
}

#[test]
fn m07_no_value_error_line() {
    let o = parse_string("[s]\nok=1\norphan\nafter=2\n");
    assert_eq!(o.error_line, 3);
    assert_eq!(o.events.len(), 2);
    assert_eq!(o.events[0].value, "1");
    assert_eq!(o.events[1].name, "after");
    assert_eq!(o.events[1].value, "2");
}

#[test]
fn m07_ini_error_line_accessor() {
    let ini = Ini::parse("[s]\nok=1\norphan\n");
    assert_eq!(ini.error_line(), 3);
}

#[test]
fn m07_success_error_line_zero() {
    let o = parse_string("[s]\na=1\n");
    assert_eq!(o.error_line, 0);
}

#[test]
fn m07_first_error_wins_line() {
    // two errors; first error line should stick (STOP_ON_FIRST_ERROR=0 but first error recorded)
    let o = parse_string("[ok]\na=1\n[bad\nx=1\norphan2\n");
    assert_eq!(o.error_line, 3);
}

#[test]
fn m07_section_spaces_not_error() {
    let o = parse_string("[ section 2 ]\nhappy  =  4\nsad =\n");
    assert_eq!(o.error_line, 0);
    assert_eq!(o.events[0].section, " section 2 ");
    assert_eq!(o.events[0].value, "4");
    assert_eq!(o.events[1].value, "");
}
