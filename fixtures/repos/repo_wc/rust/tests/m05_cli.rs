//! Milestone m05: count_file + CLI binary behavior.
use std::io::Write;
use std::process::Command;
use wcapp::{count_file, format_counts};

#[test]
fn m05_file_counts() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("wcapp_test_{}.txt", std::process::id()));
    {
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "alpha beta\ngamma\n").unwrap();
    }
    let got = count_file(path.to_str().unwrap()).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(got, (2, 3, 17));
}

#[test]
fn m05_format_matches_file() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("wcapp_fmt_{}.txt", std::process::id()));
    {
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "hello world\n").unwrap();
    }
    let p = path.to_str().unwrap();
    let (l, w, b) = count_file(p).unwrap();
    let formatted = format_counts(l, w, b, "label");
    let _ = std::fs::remove_file(&path);
    assert_eq!(formatted, "1 2 12 label");
}

#[test]
fn m05_cli_prints_counts() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("wcapp_cli_{}.txt", std::process::id()));
    {
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "hello world\nfoo bar baz\n").unwrap();
    }
    let bin = env!("CARGO_BIN_EXE_wcapp");
    let out = Command::new(bin)
        .arg(&path)
        .output()
        .expect("run wcapp");
    let _ = std::fs::remove_file(&path);
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.trim_end();
    // "<lines> <words> <bytes> <path>"
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    assert_eq!(parts.len(), 4, "got {line:?}");
    assert_eq!(parts[0], "2");
    assert_eq!(parts[1], "5");
    assert_eq!(parts[2], "24");
    assert_eq!(parts[3], path.to_str().unwrap());
}
