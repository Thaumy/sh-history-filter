use anyhow::{anyhow, Result};
use home::home_dir;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Not;

use crate::infra::result::IntoResult;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub output: Output,
    pub predicate: Predicate,
}

#[derive(Deserialize)]
pub struct Output {
    pub dedup: bool,
}

#[derive(Deserialize)]
pub struct Predicate {
    pub regex: HashSet<String>,
}

const DEFAULT_CFG: &str = r#"[output]
dedup = true

[predicate]
regex = []
"#;

impl Config {
    pub fn read() -> Result<Self> {
        let home_path = home_dir().ok_or_else(|| anyhow!("Can not get home dir"))?;

        let cfg_path = home_path.join("shf.toml");

        if cfg_path.exists().not() {
            let mut f = File::create(cfg_path.clone())?;
            let _ = f.write(DEFAULT_CFG.as_ref())?;
        }

        let cfg_path = fs::read_to_string(cfg_path)?;

        let cfg: Self = toml::from_str(&cfg_path).unwrap();

        cfg.into_ok()
    }
}
