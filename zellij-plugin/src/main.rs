use serde::Serialize;
use serde_json;
use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use config::Config;

mod config;
mod mode;
mod sessions;
mod ui;
mod workspaces;

#[derive(Default)]
struct State {
    config: Config,
    debug: bool,
}

#[derive(Serialize, Debug)]
struct SessionUpdate {
    name: String,
    isCurrentSession: bool,
}

#[derive(Serialize, Debug)]
struct SessionUpdateRequest {
    id: String,
    sessions: Vec<SessionUpdate>,
}

impl State {
    fn init(&mut self) {
        // self.session_manager = SessionManager::from(&self.config);
        // self.workspace_manager = WorkspaceManager::from(&self.config);
    }

    fn bootstrap(&mut self) {
        eprintln!("bootstrapping");
    }

    fn render_welcome(&mut self) {
        println!("welcome")
    }

    fn render_debug_info(&mut self) {}

    fn handle_perm_update(&mut self, _perms: PermissionStatus) -> bool {
        self.bootstrap();
        true
    }

    fn handle_key_event(
        &mut self,
        key: KeyWithModifier,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match key.bare_key {
            // BareKey::Esc => {
            //     close_self();
            //
            //     Ok(false)
            // }
            // BareKey::Char('q') => {
            //     self.debug = !self.debug;
            //     Ok(true)
            // }
            // BareKey::Char('c') => {
            //     eprintln!("Running command");
            //     run_command(&["ls"], BTreeMap::new());
            //
            //     Ok(true)
            // }
            // BareKey::Up | BareKey::Char('k') => {
            //     self.picker.handle_up();
            //     Ok(true)
            // }
            // BareKey::Down | BareKey::Char('j') => {
            //     self.picker.handle_down();
            //     Ok(true)
            // }
            // BareKey::Char('r') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
            //     self.reload();
            //     Ok(false)
            // }
            // BareKey::Char('d') if key.has_modifiers(&[KeyModifier::Shift, KeyModifier::Ctrl]) => {
            //     self.session_manager.clear_dead_sessions();
            //
            //     Ok(false)
            // }
            // BareKey::Enter => {
            //     if let Some(selected) = self.picker.get_selection() {
            //         let sessions = self.session_manager.list_all_sessions();
            //         self.workspace_manager
            //             .activate_workspace(&selected, sessions);
            //     }
            //
            //     close_self();
            //     Ok(false)
            // }
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
            PermissionType::WebAccess,
        ]);

        subscribe(&[
            EventType::Key,
            EventType::FileSystemUpdate,
            EventType::SessionUpdate,
            EventType::RunCommandResult,
            EventType::WebRequestResult,
            EventType::PermissionRequestResult,
            EventType::ConfigWasWrittenToDisk,
            EventType::FailedToWriteConfigToDisk,
            EventType::HostFolderChanged,
            EventType::FailedToChangeHostFolder,
            EventType::WebRequestResult,
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
                let session_updates: Vec<SessionUpdate> = sessions
                    .iter()
                    .map(|session| SessionUpdate {
                        name: session.name.clone(),
                        isCurrentSession: session.is_current_session,
                    })
                    .collect();

                let req = SessionUpdateRequest {
                    id: String::from("test string"),
                    sessions: session_updates,
                };

                let body = serde_json::to_vec(&req).unwrap();
                let context = BTreeMap::new();
                web_request(
                    String::from("http://localhost:3333/zellij/sessions"),
                    HttpVerb::Put,
                    BTreeMap::new(),
                    body,
                    context,
                );
            }
            Event::Key(key) => match self.handle_key_event(key) {
                Ok(rerender_needed) => should_render = rerender_needed,
                Err(e) => {
                    eprintln!("Failed to handle keypress: {:#?}", e);
                    should_render = false;
                }
            },
            Event::WebRequestResult(_status, _headers, raw_body, _context) => unsafe {
                let body = String::from_utf8_unchecked(raw_body);
                eprintln!("Resp body: {:#?}", body)
            },
            _ => {
                eprintln!("Unhandled event: {:#?}", event)
            }
        }

        should_render
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        eprintln!("Received pipe: {:#?}", pipe_message);
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        if self.debug {
            self.render_debug_info();
        }
        self.render_welcome()
    }
}
