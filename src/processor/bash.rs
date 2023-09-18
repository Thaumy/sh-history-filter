use regex::Regex;
use std::collections::{HashSet};
use std::ops::Not;
use crate::ordered_hash_set::OrderedHashSet;

pub fn filter(history: &str, regex_set: HashSet<Regex>) -> String {
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
