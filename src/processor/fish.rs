use regex::Regex;
use serde::{Deserialize, Serialize};
use std::ops::Not;

#[derive(Serialize, Deserialize)]
struct Entry {
    pub cmd: String,
    pub when: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
}

pub fn filter(history: &str, regex_set: &[Regex]) -> String {
    let entry_vec: Vec<Entry> =
        serde_yaml::from_str(history).unwrap_or_else(|_| panic!("Can not deserialize history"));
    let entry_vec = entry_vec
        .into_iter()
        .filter(|entry| regex_set.iter().any(|r| r.is_match(&entry.cmd)).not())
        .collect::<Vec<Entry>>();
    serde_yaml::to_string(&entry_vec).unwrap_or_else(|_| panic!("Can not serialize to history"))
}

#[test]
fn test_filter() {
    let history = r#"- cmd: ll
  when: 1695003484
- cmd: cargo outdated -R
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

    // TODO: The format of serde_yaml output is different, but fish works well on it.
    let right = r#"- cmd: ll
  when: 1695003484
- cmd: cargo outdated -R
  when: 1695003501
- cmd: cargo add toml@0.8.0
  when: 1695003525
- cmd: vi .local/share/fish/fish_history
  when: 1695044139
  paths:
  - .local/share/fish/fish_history
"#;
    assert_eq!(left, right)
}
