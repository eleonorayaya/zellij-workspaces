use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::ROOT;

#[derive(Debug)]
pub struct Config {
    pub dirs: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dirs: vec![PathBuf::from(ROOT)],
        }
    }
}

fn parse_dirs(dirs: &str) -> Vec<PathBuf> {
    return dirs.split(';').map(PathBuf::from).collect();
}

impl From<BTreeMap<String, String>> for Config {
    fn from(config: BTreeMap<String, String>) -> Self {
        let dirs: Vec<PathBuf> = match config.get("root_dirs") {
            Some(root_dirs) => parse_dirs(root_dirs),
            _ => vec![PathBuf::from(ROOT)],
        };

        Self { dirs }
    }
}

