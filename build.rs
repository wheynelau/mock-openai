// build.rs
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    // 1. Tell Cargo to rerun this script if the sonnets or tokenizer change
    println!("cargo:rerun-if-changed=build/sonnets.txt");
    println!("cargo:rerun-if-changed=build/tokenizer.json");

    let raw_string = include_str!("build/sonnets.txt");

    let tokenizer = tokenizers::Tokenizer::from_file("build/tokenizer.json").unwrap();
    let tokens = tokenizer.encode(raw_string, false).unwrap();
    let decoded_tokens: Vec<String> = tokens
        .get_ids()
        .iter()
        .map(|id| tokenizer.decode(&[*id], true).unwrap())
        .collect();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");
    let mut f = BufWriter::new(File::create(&dest_path).unwrap());

    writeln!(f, "pub static TOKENIZED_OUTPUT: &[&str] = &[").unwrap();
    for token in &decoded_tokens {
        // We use {:?} to handle escaping quotes/newlines in the strings
        writeln!(f, "    {:?},", token).unwrap();
    }
    writeln!(f, "];").unwrap();
    writeln!(f, "pub static MAX_OUTPUT: &str = {:?};", raw_string).unwrap();
    writeln!(
        f,
        "pub static MAX_TOKENS: usize = {};",
        decoded_tokens.len()
    )
    .unwrap();
}
