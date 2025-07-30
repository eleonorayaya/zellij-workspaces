use zellij_tile::prelude::*;

use super::workspace_dir::WorkspaceDir;

#[derive(Default, Clone)]
pub struct SessionDetail {
    active: bool,
    is_workspace_dir: bool,
    pub name: String,
    pub path: String,
}

impl SessionDetail {
    pub fn new(session_info: SessionInfo, workspace_dir: Option<WorkspaceDir>) -> Self {
        let path = match workspace_dir.clone() {
            Some(dir) => dir.path,
            None => String::from(""),
        };

        Self {
            active: session_info.is_current_session,
            is_workspace_dir: workspace_dir.is_some(),
            name: session_info.name,
            path,
        }
    }

    pub fn render(&self) -> String {
        return self.name.clone();
    }

    pub fn update_from_session_info(&mut self, _session_info: SessionInfo) {}
}

impl PartialEq for SessionDetail {
    fn eq(&self, other: &SessionDetail) -> bool {
        self.name == other.name
    }
}

