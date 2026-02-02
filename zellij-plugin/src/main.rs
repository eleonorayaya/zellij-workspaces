use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use zellij_tile::prelude::*;

static LOGGER: OnceLock<Logger> = OnceLock::new();

struct Logger {
    tracing_enabled: AtomicBool,
}

impl Logger {
    fn init() -> Self {
        Logger {
            tracing_enabled: AtomicBool::new(false),
        }
    }

    fn get() -> &'static Logger {
        LOGGER.get_or_init(|| Logger::init())
    }

    fn start_tracing(&self) {
        use std::fs::File;
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        let log_path = "/host/utena.log";
        let file_result = File::create(log_path);

        match file_result {
            Ok(log_file) => {
                let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(log_file));

                if tracing_subscriber::registry()
                    .with(debug_log)
                    .try_init()
                    .is_ok()
                {
                    self.tracing_enabled.store(true, Ordering::Relaxed);
                    tracing::info!("tracing initialized at {:?}", log_path);
                } else {
                    eprintln!("[ERROR] error initializing tracing subscriber");
                }
            }
            Err(error) => {
                eprintln!("[ERROR] error creating log file: {:?}", error);
            }
        }
    }

    fn debug(&self, message: String) {
        if self.tracing_enabled.load(Ordering::Relaxed) {
            tracing::debug!("{}", message);
        } else {
            eprintln!("[DEBUG] {}", message);
        }
    }

    fn info(&self, message: String) {
        if self.tracing_enabled.load(Ordering::Relaxed) {
            tracing::info!("{}", message);
        } else {
            eprintln!("[INFO] {}", message);
        }
    }

    fn warn(&self, message: String) {
        if self.tracing_enabled.load(Ordering::Relaxed) {
            tracing::warn!("{}", message);
        } else {
            eprintln!("[WARN] {}", message);
        }
    }

    fn error(&self, message: String) {
        if self.tracing_enabled.load(Ordering::Relaxed) {
            tracing::error!("{}", message);
        } else {
            eprintln!("[ERROR] {}", message);
        }
    }
}

macro_rules! log_debug {
    ($($arg:tt)*) => {
        Logger::get().debug(format!($($arg)*))
    };
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        Logger::get().info(format!($($arg)*))
    };
}

macro_rules! log_warn {
    ($($arg:tt)*) => {
        Logger::get().warn(format!($($arg)*))
    };
}

macro_rules! log_error {
    ($($arg:tt)*) => {
        Logger::get().error(format!($($arg)*))
    };
}

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
    fn handle_key_event(
        &mut self,
        key: KeyWithModifier,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        log_debug!("Key pressed: {:?}", key);
        Ok(false)
    }

    fn launch_session_picker(&mut self) {
        if self.tui_open {
            log_debug!("Session picker already open, ignoring Ctrl+P");
            return;
        }

        log_info!("Launching session picker TUI");

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
        log_debug!("Executing command: {:?}", command);

        match command.command.as_str() {
            "open_picker" => {
                log_info!("Opening session picker via pipe command");
                self.launch_session_picker();
            }

            "switch_session" => {
                if let Some(session_name) = command.session_name {
                    log_info!("Switching to session: {}", session_name);
                    switch_session_with_cwd(Some(&session_name), None);
                    self.tui_open = false;
                } else {
                    log_error!("switch_session missing session_name");
                }
            }

            "create_session" => {
                if let (Some(session_name), Some(workspace_path)) =
                    (command.session_name, command.workspace_path)
                {
                    log_info!("Creating session: {} at {}", session_name, workspace_path);
                    let cwd = PathBuf::from(workspace_path);
                    switch_session_with_cwd(Some(&session_name), Some(cwd));
                    self.tui_open = false;
                } else {
                    log_error!("create_session missing required fields");
                }
            }

            "close_picker" => {
                log_info!("Closing session picker");
                self.tui_open = false;
            }

            _ => {
                log_warn!("Unknown command: {}", command.command);
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
            EventType::PaneClosed,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        match event {
            Event::PermissionRequestResult(perms) => {
                hide_self();

                log_debug!("Perms updated: {:#?}", perms);

                let host_path = PathBuf::from(&"/var/log");
                change_host_folder(host_path);

                should_render = false;
            }
            Event::HostFolderChanged(_host_folder) => {
                Logger::get().start_tracing();
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
                    log_error!("Failed to handle keypress: {:#?}", e);
                    should_render = false;
                }
            },
            Event::WebRequestResult(status, _headers, raw_body, _context) => unsafe {
                if status != 200 {
                    let body = String::from_utf8_unchecked(raw_body);
                    log_error!("Resp body: {:#?}", body)
                }
            },
            Event::RunCommandResult(_exit_code, _stdout, _stderr, context) => {
                if let Some(source) = context.get("source") {
                    if source == "utena-session-picker" {
                        log_info!("Session picker TUI closed (command completed)");
                        self.tui_open = false;
                    }
                }
            }
            Event::PaneClosed(_pane_id) => {
                if self.tui_open {
                    log_debug!("Pane closed, resetting TUI open state");
                    self.tui_open = false;
                }
            }
            _ => {
                log_debug!("Unhandled event: {:#?}", event)
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
                log_warn!("Received pipe message with no payload");
                return false;
            }
        };

        log_debug!("Received pipe message: {}", payload);

        let command: PluginCommand = match serde_json::from_str(payload) {
            Ok(cmd) => cmd,
            Err(e) => {
                log_error!("Failed to parse pipe command: {}", e);
                return false;
            }
        };

        self.execute_command(command);
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}
