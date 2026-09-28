//! Milestone m04: run_pipeline (parse → filter → aggregate).
use pipeline::run_pipeline;

#[test]
fn m04_basic() {
    let lines = vec![
        "# c".into(),
        "apple,3".into(),
        "banana,0".into(),
        "apple,2".into(),
        "cherry,-1".into(),
        "banana,5".into(),
    ];
    assert_eq!(
        run_pipeline(&lines),
        vec![("apple".into(), 5), ("banana".into(), 5)]
    );
}

#[test]
fn m04_empty_lines() {
    assert!(run_pipeline(&[]).is_empty());
}

#[test]
fn m04_all_filtered() {
    let lines = vec!["x,0".into(), "y,-3".into(), "#z".into()];
    assert!(run_pipeline(&lines).is_empty());
}

#[test]
fn m04_whitespace_line() {
    let lines = vec!["  ".into(), "a,1".into()];
    // blank / whitespace-only should be skipped by parse_line
    assert_eq!(run_pipeline(&lines), vec![("a".into(), 1)]);
}
