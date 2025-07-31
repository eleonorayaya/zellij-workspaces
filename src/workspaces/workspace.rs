use super::workspace_dir::WorkspaceDir;
use crate::sessions::SessionDetail;

#[derive(Clone, Debug, Default)]
pub struct Workspace<'a> {
    pub name: String,
    pub dir: WorkspaceDir,
    session: Option<&'a SessionDetail>,
}

impl From<&WorkspaceDir> for Workspace<'_> {
    fn from(dir: &WorkspaceDir) -> Self {
        Self {
            dir: dir.clone(),
            name: dir.pretty_path.clone(),
            session: None,
        }
    }
}

