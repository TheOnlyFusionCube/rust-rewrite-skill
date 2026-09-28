//! CLI: print "<lines> <words> <bytes> <path>" per argv file.
use std::env;
use std::process;
use wcapp::{count_file, format_counts};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: wcapp FILE [FILE ...]");
        process::exit(2);
    }
    for path in &args {
        match count_file(path) {
            Ok((lines, words, nbytes)) => {
                println!("{}", format_counts(lines, words, nbytes, path));
            }
            Err(e) => {
                eprintln!("{}: {}", path, e);
                process::exit(1);
            }
        }
    }
}
