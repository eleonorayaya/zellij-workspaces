use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use config::Config;
use sessions::SessionManager;

mod config;
mod sessions;

const ROOT: &str = "/host";

#[derive(Default)]
enum PluginMode {
    #[default]
    PickWorkspaceAll,
    PickWorkspaceActive,
    Welcome,
}

#[derive(Default)]
struct State {
    config: Config,
    mode: PluginMode,
    session_manager: SessionManager,
}

impl State {
    fn render_welcome(&mut self) {
        println!("welcome")
    }

    fn render_pick_workspace(&mut self, _all: bool) {
        // let workspaces = self.workspace_manager.list_workspace_dirs();
        // for space in workspaces {
        //     println!("{}", space);
        // }
        //

        for session in self.session_manager.list_all_sessions() {
            println!("{}", session.render())
        }
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from(configuration);
        self.session_manager = SessionManager::from(&self.config);
        // self.workspace_manager = WorkspaceManager::from(&self.config);

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

        self.session_manager.scan_host_dirs(&self.config);
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

