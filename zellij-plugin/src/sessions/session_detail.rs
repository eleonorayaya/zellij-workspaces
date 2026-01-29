use zellij_tile::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct SessionDetail {
    active: bool,
    pub name: String,
}

impl SessionDetail {
    pub fn new(session_info: SessionInfo) -> Self {
        Self {
            active: session_info.is_current_session,
            name: session_info.name,
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
