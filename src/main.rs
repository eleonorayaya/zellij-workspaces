use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use config::Config;
use mode::PluginMode;
use sessions::SessionManager;
use workspaces::WorkspaceManager;

mod config;
mod mode;
mod sessions;
mod workspaces;

const ROOT: &str = "/host";

#[derive(Default)]
struct State<'a> {
    config: Config,
    mode: PluginMode,
    session_manager: SessionManager,
    workspace_manager: WorkspaceManager<'a>,
}

impl State<'_> {
    fn render_welcome(&mut self) {
        println!("welcome")
    }

    fn render_pick_workspace(&mut self, _all: bool) {
        for workspace in self.workspace_manager.list_workspaces() {
            println!("Space: {:#?}", workspace);
        }
    }
}

register_plugin!(State<'static>);

impl ZellijPlugin for State<'_> {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from(configuration);
        self.session_manager = SessionManager::from(&self.config);
        self.workspace_manager = WorkspaceManager::from(&self.config);

        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::ChangeApplicationState,
            PermissionType::ReadApplicationState,
            PermissionType::FullHdAccess,
        ]);

        subscribe(&[
            EventType::Key,
            EventType::FileSystemUpdate,
            EventType::SessionUpdate,
        ]);

        self.workspace_manager.scan_host_dirs(&self.config);
        self.workspace_manager.refresh_workspaces().unwrap();
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::SessionUpdate(sessions, _) => {
                self.session_manager.update_sessions(sessions);
            }
            _ => (),
        }

        true
    }

    fn pipe(&mut self, _pipe_message: PipeMessage) -> bool {
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        match self.mode {
            PluginMode::Welcome => self.render_welcome(),
            PluginMode::PickWorkspaceActive => self.render_pick_workspace(false),
            PluginMode::PickWorkspaceAll => self.render_pick_workspace(true),
        }
    }
}

