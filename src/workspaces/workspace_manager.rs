use regex::Regex;
use std::fs::{DirEntry, exists, read_dir};
use std::path::{Component, Path, PathBuf, StripPrefixError};
use zellij_tile::prelude::*;

use super::workspace::Workspace;
use super::workspace_dir::WorkspaceDir;
use crate::config::Config;
use crate::sessions::SessionDetail;

const ROOT: &str = "/host";
const HOME_PATTERN: &str = r"/(Users|home)/[^/]*";

fn strip_leading_slash(path: &Path) -> PathBuf {
    let mut components_iter = path.components();

    // Skip the RootDir component if it's present
    if let Some(Component::RootDir) = components_iter.next() {
        // We've skipped the root, now collect the rest
        components_iter.collect()
    } else {
        // No leading slash, return a clone of the original path
        path.to_path_buf()
    }
}

fn build_host_path(path: PathBuf) -> PathBuf {
    let clean_path = strip_leading_slash(&path);
    PathBuf::from(ROOT).join(clean_path)
}

fn build_os_path(path: PathBuf) -> Result<PathBuf, StripPrefixError> {
    let stripped_path = path.strip_prefix(Path::new(ROOT))?;
    let joined_path = Path::new("/").join(stripped_path).to_path_buf();

    Ok(joined_path)
}

// TODO: home replacement stuff
// let home_regex = Regex::new(HOME_PATTERN).unwrap();
// // let cwd_str = self.cwd.to_string_lossy().into_owned();
// let pretty_cwd = home_regex.replace(&cwd_str, "~");
fn prettify_path(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

#[derive(Default)]
pub struct WorkspaceManager {
    extra_dirs: Vec<PathBuf>,
    root_dirs: Vec<PathBuf>,
    workspaces: Vec<Workspace>,
}

impl From<&Config> for WorkspaceManager {
    fn from(config: &Config) -> Self {
        Self {
            extra_dirs: config.extra_dirs.clone(),
            root_dirs: config.root_dirs.clone(),
            workspaces: vec![],
        }
    }
}

impl WorkspaceManager {
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

    pub fn activate_workspace(
        &mut self,
        workspace: &Workspace,
        active_sessions: Vec<SessionDetail>,
    ) -> Result<(), StripPrefixError> {
        let session_name = workspace.session_name();
        let cwd = build_os_path(workspace.dir.path.clone())?;

        eprintln!("Activating session {} with cwd {:#?}", session_name, cwd);

        switch_session_with_cwd(Some(&session_name), Some(cwd));

        Ok(())
    }

    // TODO: Add ignored dirs
    pub fn get_workspace_dirs(&self) -> Result<Vec<WorkspaceDir>, std::io::Error> {
        let mut results: Vec<WorkspaceDir> = vec![];

        for root in self.root_dirs.clone() {
            let dir_path = build_host_path(root);

            for entry in read_dir(dir_path)? {
                let entry = entry?;

                let file_type = entry.file_type()?;
                if !file_type.is_dir() {
                    continue;
                }

                if !self.is_git_dir(&entry)? {
                    continue;
                }

                results.push(WorkspaceDir {
                    pretty_path: prettify_path(entry.path()),
                    path: entry.path(),
                })
            }
        }

        for extra_dir in self.extra_dirs.clone() {
            let dir_path = build_host_path(extra_dir).to_owned();

            if !exists(dir_path.clone())? {
                eprintln!(
                    "Skipping nonexistent extra dir {}",
                    dir_path.to_string_lossy()
                );
                continue;
            }

            results.push(WorkspaceDir {
                pretty_path: prettify_path(dir_path.clone()),
                path: dir_path,
            })
        }

        Ok(results)
    }

    pub fn scan_host_dirs(&self, config: &Config) {
        for dir in config.root_dirs.clone() {
            let host_dir = build_host_path(dir);
            eprintln!("Scanning workspace root: {:#?}", host_dir);

            scan_host_folder(&host_dir);
        }
    }

    fn is_git_dir(&self, _entry: &DirEntry) -> Result<bool, std::io::Error> {
        Ok(true)
    }
}

