use crate::infra::option::IntoOption;
use crate::infra::result::IntoResult;
use regex::Regex;
use std::collections::HashSet;
use std::fmt::Write;
use std::ops::BitXor;
use thiserror::Error;

// WRN: fish_history file is not YAML, so we can not use serde
// Related: https://github.com/fish-shell/fish-shell/issues/4675
struct Entry {
    pub cmd: String,
    pub when: usize,
    pub paths: Option<HashSet<String>>,
}

type FmtErr = std::fmt::Error;

fn serialize(entry_vec: Vec<Entry>) -> Result<String, FmtErr> {
    let blocks = entry_vec
        .into_iter()
        .map(|entry| try {
            let mut buf = String::new();
            {
                let buf = &mut buf;
                writeln!(buf, "- cmd: {}", entry.cmd)?;
                write!(buf, "  when: {}", entry.when)?;
                if let Some(paths) = entry.paths {
                    write!(buf, "\n  paths:")?;
                    for path in paths {
                        write!(buf, "\n    - {}", path)?;
                    }
                }
            }
            buf
        })
        .collect::<Result<Vec<String>, FmtErr>>()?;

    blocks.join("\n").into_ok()
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("the line `{0}` is invalid, caused by:\n {1}")]
    BadLine(String, String),
    #[error("the context was broken when parsing line `{0}`, something may be missing")]
    BadCtx(String),
    #[error("failed to serialize, caused by:\n {0}")]
    BadFmt(FmtErr),
}

fn deserialize(history: &str) -> Result<Vec<Entry>, Error> {
    let mut lines = history.lines();

    lines.try_fold(vec![], |mut acc, line| match true {
        _ if line.starts_with("- cmd: ") => {
            acc.push(Entry {
                cmd: line[7..].to_owned(),
                when: 0,
                paths: None,
            });
            acc.into_ok()
        }
        _ if line.starts_with("  when: ") => {
            let last_block = acc
                .last_mut()
                .ok_or_else(|| Error::BadCtx(line.to_owned()))?;
            let when = {
                let when_str = &line[8..];
                str::parse::<usize>(when_str).map_err(|e| {
                    Error::BadLine(
                        line.to_owned(),
                        format!(
                            "failed to parse {}, caused by:\n {}",
                            when_str.to_owned(),
                            e
                        ),
                    )
                })?
            };
            last_block.when = when;
            acc.into_ok()
        }
        _ if line.starts_with("  paths:") => {
            let last_block = acc
                .last_mut()
                .ok_or_else(|| Error::BadCtx(line.to_owned()))?;
            last_block.paths = HashSet::new().into_some();
            acc.into_ok()
        }
        _ if line.starts_with("    - ") => {
            let last_block = acc
                .last_mut()
                .ok_or_else(|| Error::BadCtx(line.to_owned()))?;
            last_block
                .paths
                .as_mut()
                .ok_or_else(|| Error::BadCtx(line.to_owned()))?
                .insert(line[6..].to_owned());
            acc.into_ok()
        }
        _ => Error::BadLine(line.to_owned(), line.to_owned()).into_err(),
    })
}
pub fn filter(history: &str, regex_set: &[Regex], pred_rev: bool) -> Result<String, Error> {
    let entry_vec: Vec<Entry> = deserialize(history)?;
    let entry_vec = entry_vec
        .into_iter()
        .filter(|entry| {
            regex_set
                .iter()
                .any(|r| r.is_match(&entry.cmd))
                .bitxor(pred_rev)
        })
        .collect::<Vec<Entry>>();
    serialize(entry_vec).map_err(Error::BadFmt)
}

#[test]
fn test_filter_pred_rev() -> anyhow::Result<()> {
    let history = r#"- cmd: ll
  when: 1695003484
- cmd: sudo netstat -anp | awk '$5 == "LISTEN" || $5 == "CONNECTED" {count[$5]++} END {printf "Listen: %d, Conn: $d", count["LISTEN"], count["CONNECTED"]}'
  when: 1695003501
- cmd: cargo add toml@0.8.0
  when: 1695003525
- cmd: cd /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11
  when: 1678438675
  paths:
    - /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11
- cmd: ln -s /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11 rs-stdlib
  when: 1678438694
  paths:
    - /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11
- cmd: vi .local/share/fish/fish_history
  when: 1695044139
  paths:
    - .local/share/fish/fish_history"#;
    let regex_set = vec![Regex::new(r"^.* /nix/store/.+")?];
    let left = filter(history, &regex_set, true)?;

    let right = r#"- cmd: ll
  when: 1695003484
- cmd: sudo netstat -anp | awk '$5 == "LISTEN" || $5 == "CONNECTED" {count[$5]++} END {printf "Listen: %d, Conn: $d", count["LISTEN"], count["CONNECTED"]}'
  when: 1695003501
- cmd: cargo add toml@0.8.0
  when: 1695003525
- cmd: vi .local/share/fish/fish_history
  when: 1695044139
  paths:
    - .local/share/fish/fish_history"#;
    assert_eq!(left, right);

    ().into_ok()
}

#[test]
fn test_filter() -> anyhow::Result<()> {
    let history = r#"- cmd: ll
  when: 1695003484
- cmd: sudo netstat -anp | awk '$5 == "LISTEN" || $5 == "CONNECTED" {count[$5]++} END {printf "Listen: %d, Conn: $d", count["LISTEN"], count["CONNECTED"]}'
  when: 1695003501
- cmd: cargo add toml@0.8.0
  when: 1695003525
- cmd: cd /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11
  when: 1678438675
  paths:
    - /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11
- cmd: ln -s /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11 rs-stdlib
  when: 1678438694
  paths:
    - /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11
- cmd: vi .local/share/fish/fish_history
  when: 1695044139
  paths:
    - .local/share/fish/fish_history"#;
    let regex_set = vec![Regex::new(r"^.* /nix/store/.+")?];
    let left = filter(history, &regex_set, false)?;

    let right = r#"- cmd: cd /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11
  when: 1678438675
  paths:
    - /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11
- cmd: ln -s /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11 rs-stdlib
  when: 1678438694
  paths:
    - /nix/store/qjgdk4ahcg25v4fg91z3zb237gaw16dr-rust-default-1.68.0-nightly-2023-01-11"#;
    assert_eq!(left, right);

    ().into_ok()
}
