use regex::Regex;
use std::fs::{read_dir, DirEntry};
use std::path::{Path, PathBuf};
use zellij_tile::prelude::*;

use super::session_detail::SessionDetail;
use super::workspace_dir::WorkspaceDir;
use crate::config::Config;

const ROOT: &str = "/host";
const HOME_PATTERN: &str = r"/(Users|home)/[^/]*";

#[derive(Default)]
pub struct SessionManager {
    cwd: PathBuf,
    sessions: Vec<SessionDetail>,
}

impl From<&Config> for SessionManager {
    fn from(_config: &Config) -> Self {
        let cwd = get_plugin_ids().initial_cwd;

        Self {
            cwd,
            sessions: vec![],
        }
    }
}

impl SessionManager {
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

    pub fn list_all_sessions(&mut self) -> Vec<SessionDetail> {
        self.sessions.iter().cloned().collect()
    }

    pub fn update_sessions(&mut self, session_infos: Vec<SessionInfo>) {
        for session_info in session_infos {
            let existing_session = self
                .sessions
                .iter_mut()
                .find(|session| *session.name == session_info.name);

            let _ = match existing_session {
                Some(session_detail) => {
                    session_detail.update_from_session_info(session_info);
                    continue;
                }
                None => self.store_session(session_info),
            };
        }
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
                name: pretty_path,
                path: path_string_lossy,
            })
        }

        Ok(results)
    }

    fn store_session(&mut self, session_info: SessionInfo) -> Result<(), std::io::Error> {
        let workspace_dirs = self.get_workspace_dirs()?;
        let session_dir = workspace_dirs
            .into_iter()
            .find(|dir| dir.name == session_info.name);

        self.sessions
            .push(SessionDetail::new(session_info, session_dir));

        Ok(())
    }

    fn is_git_dir(&self, _entry: &DirEntry) -> Result<bool, std::io::Error> {
        Ok(true)
    }
}

