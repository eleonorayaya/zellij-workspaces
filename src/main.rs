use std::collections::BTreeMap;
use std::error;
use std::path::PathBuf;
use zellij_tile::prelude::*;

use config::Config;
use mode::PluginMode;
use sessions::SessionManager;
use ui::Picker;
use workspaces::WorkspaceManager;

mod config;
mod mode;
mod sessions;
mod ui;
mod workspaces;

const ROOT: &str = "/host";

#[derive(Default)]
struct State {
    config: Config,
    mode: PluginMode,
    picker: Picker,
    session_manager: SessionManager,
    workspace_manager: WorkspaceManager,
}

impl State {
    fn init(&mut self) {
        self.session_manager = SessionManager::from(&self.config);

        self.workspace_manager = WorkspaceManager::from(&self.config);
        self.workspace_manager.scan_host_dirs(&self.config);
        match self.workspace_manager.refresh_workspaces() {
            Err(error) => {
                eprintln!("Error refreshing workspaces: {:#?}", error);
            }
            _ => (),
        }

        self.picker = Picker::from(self.workspace_manager.list_workspaces());
    }

    fn reload(&self) {
        let plugin_id = get_plugin_ids().plugin_id;
        reload_plugin_with_id(plugin_id);
    }

    fn render_welcome(&mut self) {
        println!("welcome")
    }

    fn render_pick_workspace(&mut self, _all: bool, rows: usize, cols: usize) {
        self.picker.render(rows, cols);
    }

    fn handle_perm_update(&mut self, _perms: PermissionStatus) -> bool {
        self.init();
        true
    }

    fn handle_key_event(
        &mut self,
        key: KeyWithModifier,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match key {
            KeyWithModifier {
                bare_key: BareKey::Esc,
                key_modifiers: _,
            } => {
                // close_self();
                self.reload();

                Ok(false)
            }
            KeyWithModifier {
                bare_key: BareKey::Down,
                key_modifiers: _,
            } => {
                self.picker.handle_down();
                Ok(true)
            }
            KeyWithModifier {
                bare_key: BareKey::Up,
                key_modifiers: _,
            } => {
                self.picker.handle_up();
                Ok(true)
            }
            KeyWithModifier {
                bare_key: BareKey::Enter,
                key_modifiers: _,
            } => {
                if let Some(selected) = self.picker.get_selection() {
                    let sessions = self.session_manager.list_all_sessions();
                    self.workspace_manager
                        .activate_workspace(&selected, sessions);
                }

                Ok(false)
            }
            _ => {
                eprintln!("Key pressed: {:#?}", key);
                Ok(false)
            }
        }
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from(configuration);

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

        self.init();
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        match event {
            Event::PermissionRequestResult(perms) => {
                eprintln!("Perms updated: {:#?}", perms);
                should_render = self.handle_perm_update(perms);
            }
            Event::SessionUpdate(sessions, _) => {
                self.session_manager.update_sessions(sessions);
            }
            Event::FileSystemDelete(paths) => {
                eprintln!("Fs updated: {:#?}", paths);
            }
            Event::Key(key) => match self.handle_key_event(key) {
                Ok(rerender_needed) => should_render = rerender_needed,
                Err(e) => {
                    eprintln!("Failed to handle keypress: {:#?}", e);
                    should_render = false;
                }
            },
            _ => eprintln!("Unhandled event: {:#?}", event),
        }

        should_render
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        eprintln!("Received pipe: {:#?}", pipe_message);
        false
    }

    fn render(&mut self, rows: usize, cols: usize) {
        match self.mode {
            PluginMode::Welcome => self.render_welcome(),
            PluginMode::PickWorkspaceActive => self.render_pick_workspace(false, rows, cols),
            PluginMode::PickWorkspaceAll => self.render_pick_workspace(true, rows, cols),
        }
    }
}

