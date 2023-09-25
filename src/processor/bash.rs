use crate::infra::result::IntoResult;
use crate::ordered_hash_set::OrderedHashSet;
use regex::Regex;
use std::ops::BitXor;

/**
| regex_is_match | pred_rev | keep |
| :------------: | :------: | :--: |
| 1              | 0        | 1    |
| 0              | 0        | 0    |
| 1              | 1        | 0    |
| 0              | 1        | 1    |
=> regex_is_match ⊕ pred_rev = keep
**/
pub fn filter(history: &str, regex_set: &[Regex], pred_rev: bool) -> Result<String, !> {
    let lines = history
        .split('\n')
        .filter(|entry| regex_set.iter().any(|r| r.is_match(entry)).bitxor(pred_rev))
        .collect::<Vec<&str>>();

    lines.join("\n").into_ok()
}

pub fn dedup(history: &str) -> Result<String, !> {
    let ohs = history
        .split('\n')
        .fold(OrderedHashSet::new(), |mut acc, entry| {
            acc.insert(entry);
            acc
        });

    ohs.into_vec().join("\n").into_ok()
}

#[test]
fn test_filter_pred_rev() -> anyhow::Result<()> {
    let history = r#"echo hi
cd /nix/store/9xw1h0zihwx88jmkvaki1pzfxw0rdhvw-nixos/nixos/pkgs/servers/http/

ll /nix/store/0c3rfn378viks3z095rf99c3hfpcr13q-libcdio-2.1.0/
cd dnld
cd dnld
cd /nix/store/
nix profile remove /nix/store/bx6ayk3gb2yivjwdqzssh69v13706p31-home-manager-path
echo bye"#;
    let regex_set = vec![Regex::new(r"^.* /nix/store/.+")?, Regex::new(r"^$")?];
    let left = filter(history, &regex_set, true)?;
    println!("{}", left);

    let right = r#"echo hi
cd dnld
cd dnld
cd /nix/store/
echo bye"#;
    assert_eq!(left, right);

    ().into_ok()
}

#[test]
fn test_filter() -> anyhow::Result<()> {
    let history = r#"echo hi
cd /nix/store/9xw1h0zihwx88jmkvaki1pzfxw0rdhvw-nixos/nixos/pkgs/servers/http/

ll /nix/store/0c3rfn378viks3z095rf99c3hfpcr13q-libcdio-2.1.0/
cd dnld
cd dnld
cd /nix/store/
nix profile remove /nix/store/bx6ayk3gb2yivjwdqzssh69v13706p31-home-manager-path
echo bye"#;
    let regex_set = vec![Regex::new(r"^.* /nix/store/.+")?, Regex::new(r"^$")?];
    let left = filter(history, &regex_set, false)?;
    println!("{}", left);

    let right = r#"cd /nix/store/9xw1h0zihwx88jmkvaki1pzfxw0rdhvw-nixos/nixos/pkgs/servers/http/

ll /nix/store/0c3rfn378viks3z095rf99c3hfpcr13q-libcdio-2.1.0/
nix profile remove /nix/store/bx6ayk3gb2yivjwdqzssh69v13706p31-home-manager-path"#;
    assert_eq!(left, right);

    ().into_ok()
}

#[test]
fn test_dedup() -> anyhow::Result<()> {
    let history = r#"echo hi
cd dnld
echo hi
echo bye
cd dnld
echo hi
echo bye"#;
    let left = dedup(history)?;

    let right = r#"echo hi
cd dnld
echo bye"#;
    assert_eq!(left, right);

    ().into_ok()
}
