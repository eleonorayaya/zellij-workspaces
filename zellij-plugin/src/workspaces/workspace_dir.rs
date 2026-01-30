use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct WorkspaceDir {
    pub pretty_path: String,
    pub path: PathBuf,
}

impl PartialEq for WorkspaceDir {
    fn eq(&self, other: &WorkspaceDir) -> bool {
        self.path == other.path
    }
}
