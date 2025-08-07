use regex::Regex;
use std::fs::{DirEntry, read_dir};
use std::path::{Path, PathBuf};
use zellij_tile::prelude::*;

use super::session_detail::SessionDetail;
use crate::config::Config;

#[derive(Default)]
pub struct SessionManager {
    sessions: Vec<SessionDetail>,
}

impl From<&Config> for SessionManager {
    fn from(_config: &Config) -> Self {
        Self { sessions: vec![] }
    }
}

impl SessionManager {
    pub fn list_all_sessions(&self) -> Vec<SessionDetail> {
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

    pub fn clear_dead_sessions(&self) {
        eprintln!("Clearing dead sessions");
        delete_all_dead_sessions()
    }

    fn store_session(&mut self, session_info: SessionInfo) -> Result<(), std::io::Error> {
        self.sessions.push(SessionDetail::new(session_info));

        Ok(())
    }
}

