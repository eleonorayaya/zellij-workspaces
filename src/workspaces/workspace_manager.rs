use regex::Regex;
use std::fs::{DirEntry, read_dir};
use std::path::{Path, PathBuf};
use zellij_tile::prelude::*;

use super::workspace::Workspace;
use super::workspace_dir::WorkspaceDir;
use crate::config::Config;

const ROOT: &str = "/host";
const HOME_PATTERN: &str = r"/(Users|home)/[^/]*";

#[derive(Default)]
pub struct WorkspaceManager<'a> {
    cwd: PathBuf,
    workspaces: Vec<Workspace<'a>>,
}

impl From<&Config> for WorkspaceManager<'_> {
    fn from(_config: &Config) -> Self {
        let cwd = get_plugin_ids().initial_cwd;

        Self {
            cwd,
            workspaces: vec![],
        }
    }
}

impl WorkspaceManager<'_> {
    pub fn list_workspaces(&self) -> Vec<Workspace> {
        self.workspaces.clone()
    }

    pub fn refresh_workspaces(&mut self) -> Result<(), std::io::Error> {
        for dir in self.get_workspace_dirs()? {
            let workspace_exists = self.workspaces.iter().any(|workspace| dir == workspace.dir);

            if !workspace_exists {
                self.workspaces.push(Workspace::from(&dir));
            }
        }

        Ok(())
    }

    pub fn get_workspace_dirs(&self) -> Result<Vec<WorkspaceDir>, std::io::Error> {
        let home_regex = Regex::new(HOME_PATTERN).unwrap();
        let cwd_str = self.cwd.to_string_lossy().into_owned();
        let pretty_cwd = home_regex.replace(&cwd_str, "~");

        let mut results: Vec<WorkspaceDir> = vec![];

        for entry in read_dir(ROOT)? {
            let entry = entry?;
            let path_buf = entry.path();
            let path_string_lossy = path_buf.to_string_lossy().into_owned();
            let pretty_path = path_string_lossy.replace(ROOT, &pretty_cwd);

            let file_type = entry.file_type()?;
            if !file_type.is_dir() {
                continue;
            }

            if !self.is_git_dir(&entry)? {
                continue;
            }

            results.push(WorkspaceDir {
                pretty_path,
                path: path_string_lossy,
            })
        }

        Ok(results)
    }

    pub fn scan_host_dirs(&self, config: &Config) {
        let host = PathBuf::from(ROOT);

        for dir in config.dirs.clone() {
            let relative_path = match dir.strip_prefix(self.cwd.as_path()) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let host_path = host.join(relative_path);
            scan_host_folder(&host_path);
        }
    }

    fn is_git_dir(&self, _entry: &DirEntry) -> Result<bool, std::io::Error> {
        Ok(true)
    }
}

