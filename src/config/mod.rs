use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use notify::EventKind;
use serde::{Deserialize, Serialize};

use crate::errors;

#[derive(Serialize, Deserialize, Clone)]
pub struct Cfg {
    pub pre: String,
    pub post: String,
}

impl Cfg {
    pub fn load(path: impl Sized + AsRef<Path>) -> anyhow::Result<Self, errors::ConfigError> {
        let s = fs::read_to_string(path)?;
        let r = toml::from_str(&s)?;
        Ok(r)
    }

    pub fn str(&self) -> anyhow::Result<String, errors::ConfigError> {
        let r = toml::to_string(self)?;
        Ok(r)
    }

    pub fn save(&self, path: impl Sized + AsRef<Path>) -> anyhow::Result<(), errors::ConfigError> {
        let r = fs::write(path, self.str()?)?;
        Ok(r)
    }
}
