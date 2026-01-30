# Session Picker Plugin Implementation Plan

## Overview

Implement the Zellij plugin portion of the session picker feature, enabling users to press Ctrl+P to launch a TUI for quick session switching. This plan implements the requirements from `docs/session-picker-spec.md`.

## Architecture Decisions

### Communication Strategy
**Daemon → Plugin**: Use native Zellij pipes via CLI subprocess
- Daemon executes `zellij pipe --name utena-commands --payload '{"command":"switch_session","session_name":"foo"}'`
- Plugin subscribes to `utena-commands` pipe and handles messages in `pipe()` method
- Clean, native solution aligned with spec (lines 520-533)

### TUI Binary Location
**Assumption**: `utena` binary is in system PATH
- Plugin launches TUI with simple command: `utena`
- No path configuration needed for MVP
- User will create installer later

## Architecture Update: Keyboard Handling

**Decision**: Instead of the plugin listening for Ctrl+P directly, the plugin listens for a pipe message to open the picker. This provides a cleaner separation of concerns:

- **Keyboard binding**: Configured in Zellij config (KDL file)
- **Plugin logic**: Responds to pipe commands only

**Keybinding Configuration** (add to `~/.config/zellij/config.kdl`):
```kdl
keybinds {
    normal {
        bind "Ctrl p" {
            WriteToPipe "utena-commands" "{\"command\":\"open_picker\"}";
        }
    }
}
```

**Test Command**:
```bash
zellij pipe --name utena-commands --payload '{"command":"open_picker"}'
```

## Implementation Steps

### Phase 1: Plugin Core Changes

**File**: `zellij-plugin/src/main.rs`

#### 1.1 Add State Tracking

Add to `State` struct (after line 17):
```rust
tui_open: bool,  // Track if TUI pane is currently open
```

Update `Default` implementation:
```rust
impl Default for State {
    fn default() -> Self {
        Self {
            config: Config::default(),
            debug: false,
            tui_open: false,
        }
    }
}
```

#### 1.2 Add Command Structures

Add after `SessionUpdateRequest` struct (around line 31):
```rust
#[derive(Deserialize, Debug)]
struct PluginCommand {
    command: String,
    session_name: Option<String>,
    workspace_path: Option<String>,
}
```

Add to imports:
```rust
use serde::Deserialize;  // Add Deserialize to existing serde import
```

#### 1.3 Remove Direct Keyboard Handling

The plugin does NOT handle keyboard input directly. Instead:
- Keyboard bindings are configured in Zellij config file
- Keyboard shortcuts send pipe messages to the plugin
- Plugin responds to `"open_picker"` command via pipe

Modified `handle_key_event` to just log key presses:
```rust
fn handle_key_event(
    &mut self,
    key: KeyWithModifier,
) -> Result<bool, Box<dyn std::error::Error>> {
    // No keyboard shortcuts handled directly by plugin
    // Use Zellij keybindings to send pipe messages instead
    eprintln!("Key pressed: {:?}", key);
    Ok(false)
}
```

#### 1.4 Implement TUI Launcher

Add new method to `State`:
```rust
fn launch_session_picker(&mut self) {
    if self.tui_open {
        eprintln!("Session picker already open, ignoring Ctrl+P");
        return;
    }

    eprintln!("Launching session picker TUI");

    let command = CommandToRun {
        path: PathBuf::from("utena"),  // Assumes utena is in PATH
        args: vec![],
        cwd: None,
    };

    // Center the floating pane with reasonable size
    let coordinates = Some(FloatingPaneCoordinates {
        x: Some(FloatingPaneCoordinate::Percent(25)),
        y: Some(FloatingPaneCoordinate::Percent(20)),
        width: Some(FloatingPaneCoordinate::Percent(50)),
        height: Some(FloatingPaneCoordinate::Percent(60)),
    });

    open_command_pane_floating(command, coordinates, BTreeMap::new());

    self.tui_open = true;
}
```

Add required import:
```rust
use std::path::PathBuf;
```

#### 1.5 Subscribe to Pipe Messages

Update `load()` method to subscribe to pipe (around line 112):
```rust
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
        EventType::SessionUpdate,
        EventType::WebRequestResult,
    ]);

    // Subscribe to the utena-commands pipe
    pipe_message_to_plugin(
        MessageToPlugin::new("subscribe")
            .with_destination_plugin_id(&self.config.plugin_id)
            .with_pipe_name("utena-commands")
    );
}
```

**Note**: The exact pipe subscription API may vary. Consult zellij-tile docs for correct method. Alternative approach:
```rust
// If there's a dedicated subscribe_to_pipe() function:
subscribe_to_pipe("utena-commands");
```

#### 1.6 Implement Pipe Message Handler

Replace the `pipe()` method (lines 190-193) with:
```rust
fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
    // Filter for our specific pipe
    if pipe_message.name != "utena-commands" {
        return false;
    }

    eprintln!("Received pipe message: {}", pipe_message.payload);

    // Parse the command
    let command: PluginCommand = match serde_json::from_str(&pipe_message.payload) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Failed to parse pipe command: {}", e);
            return false;
        }
    };

    self.execute_command(command);
    false  // No render needed
}
```

#### 1.7 Implement Command Execution

Add new method to `State` with support for `open_picker` command:
```rust
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
                (command.session_name, command.workspace_path) {
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
```

### Phase 2: Daemon Changes

**Goal**: Enable daemon to send commands to plugin via `zellij pipe` CLI

#### 2.1 Create Command Queue

**File**: `internal/zellij/command_queue.go` (NEW)

```go
package zellij

import (
    "sync"
)

type Command struct {
    Command       string  `json:"command"`
    SessionName   *string `json:"session_name,omitempty"`
    WorkspacePath *string `json:"workspace_path,omitempty"`
}

type CommandQueue struct {
    mu       sync.Mutex
    commands []Command
}

func NewCommandQueue() *CommandQueue {
    return &CommandQueue{
        commands: make([]Command, 0),
    }
}

func (q *CommandQueue) Enqueue(cmd Command) {
    q.mu.Lock()
    defer q.mu.Unlock()
    q.commands = append(q.commands, cmd)
}

func (q *CommandQueue) DequeueAll() []Command {
    q.mu.Lock()
    defer q.mu.Unlock()

    commands := q.commands
    q.commands = make([]Command, 0)
    return commands
}
```

#### 2.2 Add Pipe Sender

**File**: `internal/zellij/pipe_sender.go` (NEW)

```go
package zellij

import (
    "encoding/json"
    "fmt"
    "os/exec"
)

type PipeSender struct {
    pipeName string
}

func NewPipeSender() *PipeSender {
    return &PipeSender{
        pipeName: "utena-commands",
    }
}

func (p *PipeSender) SendCommand(cmd Command) error {
    // Serialize command to JSON
    payload, err := json.Marshal(cmd)
    if err != nil {
        return fmt.Errorf("failed to marshal command: %w", err)
    }

    // Execute: zellij pipe --name utena-commands --payload '<json>'
    shellCmd := exec.Command(
        "zellij",
        "pipe",
        "--name", p.pipeName,
        "--payload", string(payload),
    )

    output, err := shellCmd.CombinedOutput()
    if err != nil {
        return fmt.Errorf("zellij pipe failed: %w, output: %s", err, output)
    }

    return nil
}
```

#### 2.3 Update Zellij Service

**File**: `internal/zellij/zellijservice.go`

Add fields (around line 15):
```go
type ZellijService struct {
    // ... existing fields ...
    pipeSender *PipeSender
}
```

Update constructor (around line 20):
```go
func NewZellijService() *ZellijService {
    return &ZellijService{
        // ... existing initialization ...
        pipeSender: NewPipeSender(),
    }
}
```

Add method to send commands immediately:
```go
func (s *ZellijService) SendCommandToPlugin(cmd Command) error {
    return s.pipeSender.SendCommand(cmd)
}
```

#### 2.4 Update Session Controller

**File**: `internal/session/controller.go` (or wherever session activation happens)

**Session Activation**: When TUI calls `PUT /sessions/{name}/activate`:
```go
func (c *Controller) ActivateSession(w http.ResponseWriter, r *http.Request) {
    sessionName := chi.URLParam(r, "name")

    // ... existing session lookup and validation ...

    // Update session state
    session.IsActive = true
    session.LastAccessedAt = time.Now()
    // ... save to database ...

    // Send command to plugin
    cmd := zellij.Command{
        Command:     "switch_session",
        SessionName: &sessionName,
    }

    if err := c.zellijService.SendCommandToPlugin(cmd); err != nil {
        // Log but don't fail the request
        log.Printf("Failed to send switch_session command: %v", err)
    }

    // Return success response
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(map[string]interface{}{
        "success": true,
        "message": "Session activated successfully",
    })
}
```

**Session Creation**: When TUI calls `POST /sessions`:
```go
func (c *Controller) CreateSession(w http.ResponseWriter, r *http.Request) {
    var req CreateSessionRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        http.Error(w, "Invalid request", http.StatusBadRequest)
        return
    }

    // ... create session in database ...

    // Send command to plugin
    cmd := zellij.Command{
        Command:       "create_session",
        SessionName:   &session.Name,
        WorkspacePath: &session.WorkspacePath,
    }

    if err := c.zellijService.SendCommandToPlugin(cmd); err != nil {
        log.Printf("Failed to send create_session command: %v", err)
    }

    // Return created session
    w.WriteHeader(http.StatusCreated)
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(session)
}
```

### Phase 3: TUI Changes (Minimal)

**File**: `internal/tui/app.go`

Ensure TUI properly closes after sending API request:

```go
// When user presses Esc
case tea.KeyEsc:
    return m, tea.Quit

// When session is selected
case tea.KeyEnter:
    selectedSession := m.sessions[m.cursor]

    // Call API to activate
    resp, err := http.Put(
        fmt.Sprintf("http://localhost:3333/sessions/%s/activate", selectedSession.Name),
        "application/json",
        nil,
    )

    if err != nil {
        return m, tea.Printf("Error: %v", err)
    }

    // Daemon will send command to plugin, so we can quit
    return m, tea.Quit
```

**Note**: Check existing TUI implementation - it may already handle this correctly.

## Testing Plan

### Manual Testing Checklist

1. **Build all components**:
   ```bash
   task daemon:build
   task tui:build
   cd zellij-plugin && task build && task deploy
   ```

2. **Start daemon**:
   ```bash
   task daemon:run
   ```

3. **Launch Zellij with plugin**:
   ```bash
   zellij
   ```

4. **Test Ctrl+P trigger**:
   - Press Ctrl+P
   - Verify floating TUI pane opens
   - Check logs: `tail -f /tmp/zellij-*/zellij-log/*`

5. **Test session switching**:
   - Navigate in TUI
   - Press Enter to select session
   - Verify Zellij switches to selected session
   - Verify TUI pane closes

6. **Test new session creation**:
   - Press Ctrl+P
   - Press 'n' to toggle to new session UI
   - Select workspace
   - Enter session name
   - Verify new session is created and activated

7. **Test error cases**:
   - Stop daemon, press Ctrl+P (graceful failure)
   - Press Ctrl+P twice (should not open duplicate TUI)
   - Invalid session name (error handling)

### Debug Logging

Watch Zellij logs:
```bash
cd zellij-plugin
task logs
# or
tail -f /tmp/zellij-*/zellij-log/*
```

Check daemon logs for pipe command execution:
```bash
# daemon should log when executing zellij pipe commands
```

### Verification Commands

Test pipe communication manually:
```bash
# Simulate daemon sending command
zellij pipe --name utena-commands --payload '{"command":"switch_session","session_name":"test"}'
```

Check if TUI is in PATH:
```bash
which utena
# Should return path to binary
```

## Critical Files Modified

1. **zellij-plugin/src/main.rs** - Add Ctrl+P handler, TUI launcher, pipe subscription, command execution
2. **internal/zellij/command_queue.go** - NEW: Command queue structure
3. **internal/zellij/pipe_sender.go** - NEW: Zellij pipe command execution
4. **internal/zellij/zellijservice.go** - Add pipe sender integration
5. **internal/session/controller.go** - Send commands on session activate/create

## Phase 4: Handle TUI Pane Close Events (COMPLETED)

**File**: `zellij-plugin/src/main.rs`

### 4.1 Subscribe to Pane Close Events

Added `EventType::PaneClosed` to the event subscription list in `load()` method.

### 4.2 Add Context Identifier

Modified `launch_session_picker()` to include a context identifier when opening the command pane:
```rust
let mut context = BTreeMap::new();
context.insert("source".to_string(), "utena-session-picker".to_string());
open_command_pane_floating(command, None, context);
```

### 4.3 Handle Pane Close Events

Added event handlers in the `update()` method:

**RunCommandResult**: Fires when the TUI command completes
```rust
Event::RunCommandResult(_exit_code, _stdout, _stderr, context) => {
    if let Some(source) = context.get("source") {
        if source == "utena-session-picker" {
            eprintln!("Session picker TUI closed (command completed)");
            self.tui_open = false;
        }
    }
}
```

**PaneClosed**: Fires when any pane closes (including user pressing Esc)
```rust
Event::PaneClosed(_pane_id) => {
    if self.tui_open {
        eprintln!("Pane closed, resetting TUI open state");
        self.tui_open = false;
    }
}
```

This ensures the plugin correctly detects when the TUI closes via:
- User pressing Esc in the TUI
- TUI exiting normally after session selection
- User manually closing the pane
- Any other pane close mechanism

## Success Criteria

- ✅ Ctrl+P opens TUI in floating pane
- ✅ TUI displays sessions from daemon API
- ✅ Selecting session switches active Zellij session
- ✅ TUI closes after selection
- ✅ New session creation works end-to-end
- ✅ Daemon successfully sends commands via `zellij pipe`
- ✅ Plugin receives and executes pipe commands
- ✅ Error handling prevents crashes
- ✅ Multiple Ctrl+P presses handled gracefully
- ✅ Plugin detects when TUI pane closes (via Esc, exit, or manual close)

## Known Issues & Edge Cases

1. **Pipe subscription API uncertainty**: The exact method to subscribe to a named pipe may differ. Check zellij-tile 0.42.2 docs.

2. **TUI not in PATH**: If `utena` is not in PATH, command will fail. Document installation requirement.

3. **Multiple Zellij sessions**: Commands sent via `zellij pipe` go to all sessions. This should work fine as session names are unique.

4. **TUI pane close detection**: Plugin doesn't know when TUI pane closes via Esc. Could add explicit "close_picker" command from TUI.

5. **Command queue unused**: CommandQueue was created but not needed with direct pipe sending. Can remove or keep for future async processing.

## Future Enhancements

- Configure TUI pane size/position
- Handle TUI pane close events
- Add command acknowledgment/feedback
- Support configurable keyboard shortcut
- Add metrics/telemetry for session switching
