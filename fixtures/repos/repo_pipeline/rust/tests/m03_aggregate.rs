//! Milestone m03: aggregate by key, sorted.
use pipeline::aggregate;

#[test]
fn m03_sum_and_sort() {
    let rows = vec![("b".into(), 1), ("a".into(), 2), ("b".into(), 3)];
    assert_eq!(aggregate(&rows), vec![("a".into(), 2), ("b".into(), 4)]);
}

#[test]
fn m03_empty() {
    assert!(aggregate(&[]).is_empty());
}

#[test]
fn m03_single() {
    assert_eq!(aggregate(&[("x".into(), 9)]), vec![("x".into(), 9)]);
}

#[test]
fn m03_three_keys() {
    let rows = vec![
        ("c".into(), 1),
        ("a".into(), 1),
        ("b".into(), 1),
        ("a".into(), 1),
    ];
    assert_eq!(
        aggregate(&rows),
        vec![("a".into(), 2), ("b".into(), 1), ("c".into(), 1)]
    );
}
