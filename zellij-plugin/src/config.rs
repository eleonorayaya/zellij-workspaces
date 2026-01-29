use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;

#[derive(Debug)]
pub struct Config {
    pub extra_dirs: Vec<PathBuf>,
    pub root_dirs: Vec<PathBuf>,
    pub initial_cwd: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            extra_dirs: vec![],
            root_dirs: vec![],
            initial_cwd: PathBuf::new(),
        }
    }
}

fn parse_dir_str(dirs: &String) -> Vec<PathBuf> {
    return dirs.split(';').map(PathBuf::from).collect();
}

fn parse_dirs(maybe_config_str: Option<&String>) -> Vec<PathBuf> {
    match maybe_config_str {
        Some(config_str) => parse_dir_str(config_str),
        _ => vec![],
    }
}

impl From<BTreeMap<String, String>> for Config {
    fn from(config: BTreeMap<String, String>) -> Self {
        eprintln!("Config: {:#?}", config);
        let root_dirs = parse_dirs(config.get("root_dirs"));
        let extra_dirs = parse_dirs(config.get("extra_dirs"));
        let initial_cwd = get_plugin_ids().initial_cwd;

        Self {
            initial_cwd: initial_cwd,
            extra_dirs: extra_dirs,
            root_dirs: root_dirs,
        }
    }
}
