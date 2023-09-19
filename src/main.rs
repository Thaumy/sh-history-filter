#![warn(clippy::all, clippy::nursery, clippy::cargo_common_metadata)]

use crate::args::{Args, ShellType};
use crate::cfg::Config;
use clap::Parser;
use regex::Regex;

pub mod args;
pub mod cfg;
pub mod ordered_hash_set;
pub mod processor;

fn main() {
    let args: Args = Args::parse();
    let history_text = args.history_text;
    let shell_type = args.shell_type;
    let cfg = Config::read().unwrap();
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
            if cfg.filter.remove_empty_line {
                history = processor::bash::remove_empty_line(&history)
            }
            history
        }
        ShellType::Fish => processor::fish::filter(&history_text, &regex_set),
    };

    println!("{}", history);
}
