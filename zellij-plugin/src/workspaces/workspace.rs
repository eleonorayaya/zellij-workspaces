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

impl Workspace {
    pub fn session_name(&self) -> String {
        match self.dir.path.components().last() {
            Some(component) => component.as_os_str().to_string_lossy().to_string(),
            _ => String::from("no name"),
        }
    }
}
