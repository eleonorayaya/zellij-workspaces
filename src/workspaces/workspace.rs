use super::workspace_dir::WorkspaceDir;

#[derive(Clone, Debug, Default)]
pub struct Workspace {
    pub name: String,
    pub dir: WorkspaceDir,
}

impl From<&WorkspaceDir> for Workspace {
    fn from(dir: &WorkspaceDir) -> Self {
        Self {
            dir: dir.clone(),
            name: dir.pretty_path.clone(),
        }
    }
}

