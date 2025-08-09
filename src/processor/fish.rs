use crate::infra::option::IntoOption;
use crate::infra::result::IntoResult;
use crate::infra::vec::VecExt;
use regex::Regex;
use std::collections::HashSet;
use std::fmt::Write;
use std::ops::BitXor;
use thiserror::Error;

// WRN: fish_history file is not YAML, so we can not use serde
// Related: https://github.com/fish-shell/fish-shell/issues/4675
#[derive(Debug)]
struct Entry {
    pub cmd: String,
    pub when: usize,
    pub paths: Option<HashSet<String>>,
}

type FmtErr = std::fmt::Error;

fn serialize(entry_vec: Vec<Entry>) -> Result<String, FmtErr> {
    let mut serialized = entry_vec.into_iter().try_fold::<_, _, Result<_, _>>(
        String::new(),
        |mut acc, entry| try {
            {
                let acc = &mut acc;
                writeln!(acc, "- cmd: {}", entry.cmd)?;
                writeln!(acc, "  when: {}", entry.when)?;
                if let Some(paths) = entry.paths {
                    let paths = paths.into_iter().try_fold(String::new(), |mut acc, path| {
                        writeln!(&mut acc, "    - {}", path).map(|_| acc)
                    })?;
                    write!(acc, "  paths:\n{}", paths)?;
                }
            }
            acc
        },
    )?;
    // remove the trailing '\n'
    serialized.pop();
    serialized.into_ok()
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("the line `{0}` is invalid, caused by:\n {1}")]
    BadLine(String, String),
    #[error("the context was broken when parsing line `{0}`, something may be missing")]
    BadCtx(String),
    #[error("failed to serialize, caused by:\n {0}")]
    BadFmt(FmtErr),
}

fn deserialize(history: &str) -> Result<Vec<Entry>, Error> {
    let prefix_cmd = "- cmd: ";
    let prefix_when = "  when: ";
    let prefix_paths = "  paths:";
    let prefix_paths_item = "    - ";
    history
        .lines()
        .try_fold(vec![], |mut acc, line| match true {
            _ if line.starts_with(prefix_cmd) => acc
                .chain_push(Entry {
                    cmd: line[prefix_cmd.len()..].to_owned(),
                    when: 0,
                    paths: None,
                })
                .into_ok(),
            _ if line.starts_with(prefix_when) => {
                let last_block = acc
                    .last_mut()
                    .ok_or_else(|| Error::BadCtx(line.to_owned()))?;
                let when = {
                    let when_str = &line[prefix_when.len()..];
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
            _ if line.starts_with(prefix_paths) => {
                let last_block = acc
                    .last_mut()
                    .ok_or_else(|| Error::BadCtx(line.to_owned()))?;
                last_block.paths = HashSet::new().into_some();
                acc.into_ok()
            }
            _ if line.starts_with(prefix_paths_item) => {
                let last_block = acc
                    .last_mut()
                    .ok_or_else(|| Error::BadCtx(line.to_owned()))?;
                last_block
                    .paths
                    .as_mut()
                    .ok_or_else(|| Error::BadCtx(line.to_owned()))?
                    .insert(line[prefix_paths_item.len()..].to_owned());
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
