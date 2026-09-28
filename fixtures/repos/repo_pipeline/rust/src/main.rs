//! CLI: pipeline INPUT OUTPUT
use std::env;
use std::process;
use pipeline::{read_rows, run_pipeline, write_aggregates};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: pipeline INPUT OUTPUT");
        process::exit(2);
    }
    let lines = match read_rows(&args[0]) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    let items = run_pipeline(&lines);
    if let Err(e) = write_aggregates(&args[1], &items) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
