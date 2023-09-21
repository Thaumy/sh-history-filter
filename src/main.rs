#![warn(clippy::all, clippy::nursery, clippy::cargo_common_metadata)]

use crate::args::{Args, ShellType};
use crate::cfg::Config;
use anyhow::Result;
use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::Path;

pub mod args;
pub mod cfg;
pub mod ordered_hash_set;
pub mod processor;

fn main() -> Result<()> {
    let args: Args = Args::parse();
    let cfg = Config::read().unwrap();
    let history_text = fs::read_to_string(Path::new(&args.history_path))?;
    let shell_type = args.shell;
    let regex_set: Vec<_> = cfg
        .filter
        .regex
        .iter()
        .map(|r| Regex::new(r).unwrap())
        .collect();

    let history = match shell_type {
        ShellType::Bash => {
            let mut history = processor::bash::filter(&history_text, &regex_set);
            if cfg.filter.dedup {
                history = processor::bash::dedup(&history)
            }
            history
        }
        ShellType::Fish => processor::fish::filter(&history_text, &regex_set),
    };

    println!("{}", history);

    Ok(())
}
