//! Milestone m02: filter_rows (value > 0).
use pipeline::filter_rows;

#[test]
fn m02_keeps_positive() {
    let rows = vec![
        ("a".into(), 1),
        ("b".into(), 0),
        ("c".into(), -2),
        ("d".into(), 5),
    ];
    assert_eq!(
        filter_rows(&rows),
        vec![("a".into(), 1), ("d".into(), 5)]
    );
}

#[test]
fn m02_empty() {
    assert!(filter_rows(&[]).is_empty());
}

#[test]
fn m02_all_nonpositive() {
    let rows = vec![("a".into(), 0), ("b".into(), -1)];
    assert!(filter_rows(&rows).is_empty());
}

#[test]
fn m02_preserves_order() {
    let rows = vec![("z".into(), 2), ("a".into(), 1)];
    assert_eq!(
        filter_rows(&rows),
        vec![("z".into(), 2), ("a".into(), 1)]
    );
}
