#[derive(Clone, Default)]
pub enum PluginMode {
    #[default]
    PickWorkspaceAll,
    PickWorkspaceActive,
    Welcome,
}

