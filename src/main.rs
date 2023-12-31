#![feature(never_type)]
#![feature(try_blocks)]
#![warn(clippy::all, clippy::nursery, clippy::cargo_common_metadata)]

use crate::args::{Args, ShellType};
use crate::cfg::Config;
use crate::infra::result::IntoResult;
use anyhow::Result;
use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::Path;

pub mod args;
pub mod cfg;
pub mod infra;
pub mod ordered_set;
pub mod processor;

fn main() -> Result<()> {
    let args: Args = Args::parse();
    let cfg = Config::read().unwrap();
    let history_text = fs::read_to_string(Path::new(&args.history_path))?;
    let shell_type = args.shell;
    let regex_set = cfg
        .predicate
        .regex
        .iter()
        .map(|r| Regex::new(r).unwrap())
        .collect::<Vec<_>>();

    let history = match shell_type {
        ShellType::Bash => {
            let mut history = processor::bash::filter(&history_text, &regex_set, args.pred_rev)?;
            if cfg.output.dedup {
                history = processor::bash::dedup(&history)?
            }
            history
        }
        ShellType::Fish => processor::fish::filter(&history_text, &regex_set, args.pred_rev)?,
    };

    println!("{}", history);

    ().into_ok()
}
