use std::collections::BTreeMap;
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

    fn render_pick_workspace(&mut self, _all: bool, rows: usize, cols: usize) {
        let picker = Picker::from(self.workspace_manager.list_workspaces());
        picker.render(rows, cols);
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
            Event::Key(key) => {
                match key {
                    KeyWithModifier {
                        bare_key: BareKey::Esc,
                        key_modifiers: _,
                    } => {
                        close_self();
                    }
                    // KeyWithModifier {
                    //     bare_key: BareKey::Down,
                    //     key_modifiers: _,
                    // } => {
                    //     self.dirlist.handle_down();
                    // }
                    // KeyWithModifier {
                    //     bare_key: BareKey::Up,
                    //     key_modifiers: _,
                    // } => {
                    //     self.dirlist.handle_up();
                    // }
                    // KeyWithModifier {
                    //     bare_key: BareKey::Enter,
                    //     key_modifiers: _,
                    // } => {
                    //     if let Some(selected) = self.dirlist.get_selected() {
                    //         let _ = self.switch_session_with_cwd(Path::new(&selected));
                    //         close_self();
                    //     }
                    // }
                    // KeyWithModifier {
                    //     bare_key: BareKey::Backspace,
                    //     key_modifiers: _,
                    // } => {
                    //     self.textinput.handle_backspace();
                    //     self.dirlist
                    //         .set_search_term(self.textinput.get_text().as_str());
                    // }
                    // KeyWithModifier {
                    //     bare_key: BareKey::Char(c),
                    //     key_modifiers: _,
                    // } => {
                    //     self.textinput.handle_char(c);
                    //     self.dirlist
                    //         .set_search_term(self.textinput.get_text().as_str());
                    // }
                    _ => eprintln!("Key pressed: {:#?}", key),
                }
            }
            _ => (),
        }

        true
    }

    fn pipe(&mut self, _pipe_message: PipeMessage) -> bool {
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

