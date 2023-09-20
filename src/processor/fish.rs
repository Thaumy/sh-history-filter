use regex::Regex;
use std::collections::HashSet;
use std::ops::Not;

// WRN: fish_history file is not YAML, so we can not use serde
// Related: https://github.com/fish-shell/fish-shell/issues/4675
struct Entry {
    pub cmd: String,
    pub when: usize,
    pub paths: Option<HashSet<String>>,
}

fn serialize(entry_vec: Vec<Entry>) -> String {
    entry_vec
        .into_iter()
        .map(|entry| {
            let mut string = String::new();
            string.push_str(&format!("- cmd: {}\n", entry.cmd));
            string.push_str(&format!("  when: {}", entry.when));
            if let Some(paths) = entry.paths {
                string.push_str("\n  paths:");
                paths.into_iter().for_each(|path| {
                    string.push_str(&format!("\n    - {}", path));
                });
            }
            string
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn deserialize(history: &str) -> Vec<Entry> {
    history.lines().fold(vec![], |mut acc, line| match true {
        _ if line.starts_with("- cmd: ") => {
            acc.push(Entry {
                cmd: line[7..].to_owned(),
                when: 0,
                paths: None,
            });
            acc
        }
        _ if line.starts_with("  when: ") => {
            acc.last_mut().unwrap().when = str::parse::<usize>(&line[8..])
                .unwrap_or_else(|e| panic!("parse `when` to usize failed: {}", e));
            acc
        }
        _ if line.starts_with("  paths:") => {
            acc.last_mut().unwrap().paths = Some(HashSet::new());
            acc
        }
        _ if line.starts_with("    - ") => {
            acc.last_mut()
                .unwrap()
                .paths
                .as_mut()
                .expect("`path` is None, unable to insert")
                .insert(line[6..].to_owned());
            acc
        }
        _ => panic!("Invalid line format: {}", line),
    })
}

pub fn filter(history: &str, regex_set: &[Regex]) -> String {
    let entry_vec: Vec<Entry> = deserialize(history);
    let entry_vec = entry_vec
        .into_iter()
        .filter(|entry| regex_set.iter().any(|r| r.is_match(&entry.cmd)).not())
        .collect::<Vec<Entry>>();
    serialize(entry_vec)
}

#[test]
fn test_filter() {
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
    let regex_set = vec![Regex::new(r"^.* /nix/store/.+").unwrap()];
    let left = filter(history, &regex_set);
    println!("{}", left);

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
    assert_eq!(left, right)
}
