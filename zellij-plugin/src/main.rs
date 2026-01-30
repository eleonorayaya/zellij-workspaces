use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    tui_open: bool,
}

#[derive(Serialize, Debug)]
struct SessionUpdate {
    name: String,
    is_current_session: bool,
}

#[derive(Serialize, Debug)]
struct SessionUpdateRequest {
    id: String,
    sessions: Vec<SessionUpdate>,
}

#[derive(Deserialize, Debug)]
struct PluginCommand {
    command: String,
    session_name: Option<String>,
    workspace_path: Option<String>,
}

impl State {
    fn render_welcome(&mut self) {
        println!("welcome")
    }

    fn handle_key_event(
        &mut self,
        key: KeyWithModifier,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        eprintln!("Key pressed: {:?}", key);
        Ok(false)
    }

    fn launch_session_picker(&mut self) {
        if self.tui_open {
            eprintln!("Session picker already open, ignoring Ctrl+P");
            return;
        }

        eprintln!("Launching session picker TUI");

        let command = CommandToRun {
            path: PathBuf::from("utena"),
            args: vec![],
            cwd: None,
        };

        let mut context = BTreeMap::new();
        context.insert("source".to_string(), "utena-session-picker".to_string());

        open_command_pane_floating(command, None, context);

        self.tui_open = true;
    }

    fn execute_command(&mut self, command: PluginCommand) {
        eprintln!("Executing command: {:?}", command);

        match command.command.as_str() {
            "open_picker" => {
                eprintln!("Opening session picker via pipe command");
                self.launch_session_picker();
            }

            "switch_session" => {
                if let Some(session_name) = command.session_name {
                    eprintln!("Switching to session: {}", session_name);
                    switch_session_with_cwd(Some(&session_name), None);
                    self.tui_open = false;
                } else {
                    eprintln!("switch_session missing session_name");
                }
            }

            "create_session" => {
                if let (Some(session_name), Some(workspace_path)) =
                    (command.session_name, command.workspace_path)
                {
                    eprintln!("Creating session: {} at {}", session_name, workspace_path);
                    let cwd = PathBuf::from(workspace_path);
                    switch_session_with_cwd(Some(&session_name), Some(cwd));
                    self.tui_open = false;
                } else {
                    eprintln!("create_session missing required fields");
                }
            }

            "close_picker" => {
                eprintln!("Closing session picker");
                self.tui_open = false;
            }

            _ => {
                eprintln!("Unknown command: {}", command.command);
            }
        }
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::ChangeApplicationState,
            PermissionType::ReadApplicationState,
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
            EventType::PaneClosed,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        match event {
            Event::PermissionRequestResult(perms) => {
                eprintln!("Perms updated: {:#?}", perms);
                should_render = false;
            }
            Event::SessionUpdate(sessions, _) => {
                let session_updates: Vec<SessionUpdate> = sessions
                    .iter()
                    .map(|session| SessionUpdate {
                        name: session.name.clone(),
                        is_current_session: session.is_current_session,
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
            Event::WebRequestResult(status, _headers, raw_body, _context) => unsafe {
                if status != 200 {
                    let body = String::from_utf8_unchecked(raw_body);
                    eprintln!("Resp body: {:#?}", body)
                }
            },
            Event::RunCommandResult(_exit_code, _stdout, _stderr, context) => {
                if let Some(source) = context.get("source") {
                    if source == "utena-session-picker" {
                        eprintln!("Session picker TUI closed (command completed)");
                        self.tui_open = false;
                    }
                }
            }
            Event::PaneClosed(_pane_id) => {
                if self.tui_open {
                    eprintln!("Pane closed, resetting TUI open state");
                    self.tui_open = false;
                }
            }
            _ => {
                eprintln!("Unhandled event: {:#?}", event)
            }
        }

        should_render
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if pipe_message.name != "utena-commands" {
            return false;
        }

        let payload = match &pipe_message.payload {
            Some(p) => p,
            None => {
                eprintln!("Received pipe message with no payload");
                return false;
            }
        };

        eprintln!("Received pipe message: {}", payload);

        let command: PluginCommand = match serde_json::from_str(payload) {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Failed to parse pipe command: {}", e);
                return false;
            }
        };

        self.execute_command(command);
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        self.render_welcome()
    }
}
