use crate::ordered_hash_set::OrderedHashSet;
use regex::Regex;
use std::ops::Not;

pub fn filter(history: &str, regex_set: &[Regex]) -> String {
    history
        .split('\n')
        .filter(|entry| regex_set.iter().any(|r| r.is_match(entry)).not())
        .collect::<Vec<&str>>()
        .join("\n")
}

pub fn dedup(history: &str) -> String {
    let ohs = OrderedHashSet::new();

    history
        .split('\n')
        .fold(ohs, |mut acc, entry| {
            acc.insert(entry);
            acc
        })
        .into_vec()
        .join("\n")
}

pub fn remove_empty_line(history: &str) -> String {
    history.replace("\n\n", "\n")
}

#[test]
fn test_filter() {
    let history = r#"echo hi
cd /nix/store/9xw1h0zihwx88jmkvaki1pzfxw0rdhvw-nixos/nixos/pkgs/servers/http/
ll /nix/store/0c3rfn378viks3z095rf99c3hfpcr13q-libcdio-2.1.0/
cd dnld
cd /nix/store/
nix profile remove /nix/store/bx6ayk3gb2yivjwdqzssh69v13706p31-home-manager-path
echo bye"#;
    let regex_set = vec![Regex::new(r"^.* /nix/store/.+").unwrap()];
    let left = filter(history, &regex_set);

    let right = r#"echo hi
cd dnld
cd /nix/store/
echo bye"#;
    assert_eq!(left, right)
}

#[test]
fn test_dedup() {
    let history = r#"echo hi
cd dnld
echo hi
echo bye
cd dnld
echo hi
echo bye"#;
    let left = dedup(history);

    let right = r#"echo hi
cd dnld
echo bye"#;
    assert_eq!(left, right)
}

#[test]
fn test_remove_empty_line() {
    let history = r#"echo hi

cd dnld

echo bye"#;
    let left = remove_empty_line(history);

    let right = r#"echo hi
cd dnld
echo bye"#;
    assert_eq!(left, right)
}
