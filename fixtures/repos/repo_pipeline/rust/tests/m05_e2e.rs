//! Milestone m05: read_rows / write_aggregates + CLI.
use std::io::Write;
use std::process::Command;
use pipeline::{read_rows, run_pipeline, write_aggregates};

#[test]
fn m05_e2e_files() {
    let dir = std::env::temp_dir();
    let pid = std::process::id();
    let inp = dir.join(format!("pipe_in_{}.csv", pid));
    let outp = dir.join(format!("pipe_out_{}.csv", pid));
    {
        let mut f = std::fs::File::create(&inp).unwrap();
        write!(
            f,
            "# sample\napple,3\nbanana,0\napple,2\ncherry,-1\nbanana,5\ndate,4\nbogus\ncherry,1\n"
        )
        .unwrap();
    }
    let lines = read_rows(inp.to_str().unwrap()).unwrap();
    let items = run_pipeline(&lines);
    write_aggregates(outp.to_str().unwrap(), &items).unwrap();
    let got = std::fs::read_to_string(&outp).unwrap();
    let _ = std::fs::remove_file(&inp);
    let _ = std::fs::remove_file(&outp);
    assert_eq!(got, "apple,5\nbanana,5\ncherry,1\ndate,4\n");
}

#[test]
fn m05_write_empty() {
    let dir = std::env::temp_dir();
    let outp = dir.join(format!("pipe_empty_{}.csv", std::process::id()));
    write_aggregates(outp.to_str().unwrap(), &[]).unwrap();
    let got = std::fs::read_to_string(&outp).unwrap();
    let _ = std::fs::remove_file(&outp);
    assert_eq!(got, "");
}

#[test]
fn m05_cli() {
    let dir = std::env::temp_dir();
    let pid = std::process::id();
    let inp = dir.join(format!("pipe_cli_in_{}.csv", pid));
    let outp = dir.join(format!("pipe_cli_out_{}.csv", pid));
    {
        let mut f = std::fs::File::create(&inp).unwrap();
        write!(f, "a,2\nb,0\na,3\n").unwrap();
    }
    let bin = env!("CARGO_BIN_EXE_pipeline");
    let status = Command::new(bin)
        .arg(&inp)
        .arg(&outp)
        .status()
        .expect("run pipeline");
    assert!(status.success());
    let got = std::fs::read_to_string(&outp).unwrap();
    let _ = std::fs::remove_file(&inp);
    let _ = std::fs::remove_file(&outp);
    assert_eq!(got, "a,5\n");
}
