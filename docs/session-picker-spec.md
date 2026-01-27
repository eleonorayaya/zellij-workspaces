# Session Picker Feature Specification

## 1. Feature Overview

The Session Picker is a core feature that enables users to quickly switch between existing Zellij sessions and create new ones. It provides a streamlined keyboard-driven interface that integrates seamlessly with Zellij's workflow.

**Key Capabilities:**
- Quick access via keyboard shortcut (Ctrl+P)
- View all available sessions sorted by most-recently-used
- Switch to any existing session with minimal keystrokes
- Create new sessions from a list of available workspaces
- Vim-style keyboard navigation throughout

**Activation Method:**
The feature is triggered by pressing Ctrl+P from within any Zellij session, which launches a floating pane containing the TUI.

**Two Primary Modes:**
1. **Session List**: Browse and select from existing sessions
2. **New Session**: Create a new session by selecting a workspace and providing a name

---

## 2. User Experience Flow

### Session List Flow

1. **Trigger**: User presses Ctrl+P in any Zellij session
2. **Launch**: Floating pane opens with TUI displaying the Session List
3. **Display**: Sessions are shown in most-recently-used order
4. **Navigate**: User moves up/down with j/k keys (vim-style)
5. **Select**: User presses Enter → switches to the selected session, TUI pane dismisses
6. **Toggle**: User presses 'n' → transitions to New Session UI
7. **Exit**: User presses Esc or q → dismisses TUI without taking action

### New Session Flow

1. **Entry**: User in New Session UI (accessed via 'n' from Session List)
2. **Display**: Shows list of available workspaces (directories)
3. **Navigate**: User moves up/down with j/k keys
4. **Select Workspace**: User presses Enter → advances to name input prompt
5. **Name Input**: User types session name or leaves blank for default
6. **Create**: User presses Enter → new session is created and activated, TUI dismisses
7. **Return**: User presses Esc → returns to Session List (cancel action)

---

## 3. Component Architecture

### Daemon (Go)

**Primary Responsibilities:**
- Maintain authoritative session registry with metadata (name, last accessed time, workspace path, active status)
- Persist session history to disk for durability across restarts
- Manage workspace discovery via WorkspaceManager
- Communicate with Zellij plugin to execute session operations

**API Responsibilities:**
- `GET /sessions`: List all sessions sorted by recency
- `POST /sessions`: Create new session with name and workspace
- `PUT /sessions/{name}/activate`: Mark session as active and trigger switch
- `GET /workspaces`: Return available workspace directories
- `PUT /zellij/sessions`: Receive and process session updates from plugin

**State Management:**
- In-memory map for fast session lookups
- Mutex-protected for concurrent access
- JSON persistence to `~/.config/utena/sessions.json`
- Load from disk on startup, save on updates

### TUI (Go + Bubbletea)

**Primary Responsibilities:**
- Implement Session List view (interactive picker with vim navigation)
- Implement New Session view (workspace picker + name input screen)
- Handle all keyboard input events (navigation, selection, mode switching)
- Communicate with daemon API for data fetching and state mutations
- Self-terminate after successful session action

**UI Components:**
- `SessionListModel`: Display sessions, handle navigation and selection
- `NewSessionModel`: Display workspaces, handle navigation and selection
- `NameInputModel`: Text input for session name with validation
- `App`: Parent model managing view routing and lifecycle

**Interaction Patterns:**
- Async API calls via `tea.Cmd`
- Message-based state updates
- Graceful error handling with user feedback

### Zellij Plugin (Rust/WASM)

**Primary Responsibilities:**
- Listen for Ctrl+P keyboard combo in Zellij event loop
- Launch floating pane with TUI binary when triggered
- Send session update events to daemon (session created, switched, closed)
- Subscribe to Zellij pipe for daemon commands
- Execute session operations via Zellij API (switch session, create session)

**Communication:**
- Outbound: HTTP PUT requests to daemon with session updates
- Inbound: Subscribe to `utena-commands` pipe for daemon instructions
- Command format: JSON messages like `{"command": "switch_session", "session_name": "foo"}`

**Integration Points:**
- Use Zellij's `SessionUpdate` event for session state changes
- Use Zellij's `open_command_pane()` to launch TUI
- Use Zellij's `switch_session()` API to activate sessions

---

## 4. Data Structures

### Session (daemon-side)

```go
type Session struct {
    Name            string    `json:"name"`             // User-provided or generated name
    WorkspacePath   string    `json:"workspace_path"`   // Working directory for session
    LastAccessedAt  time.Time `json:"last_accessed_at"` // For MRU sorting
    IsActive        bool      `json:"is_active"`        // Currently active in Zellij
    CreatedAt       time.Time `json:"created_at"`       // Session creation timestamp
}
```

### Workspace (daemon-side)

```go
type Workspace struct {
    Path      string `json:"path"`       // Full filesystem path
    Name      string `json:"name"`       // Display name (last path component)
    IsGitRepo bool   `json:"is_git_repo"` // Optional: detect git repositories
}
```

### API Request/Response Types

```go
// GET /sessions response
type SessionListResponse struct {
    Sessions []Session `json:"sessions"`
}

// POST /sessions request
type CreateSessionRequest struct {
    Name          string `json:"name"`
    WorkspacePath string `json:"workspace_path"`
}

// POST /sessions response
type CreateSessionResponse struct {
    Session Session `json:"session"`
}

// PUT /sessions/{name}/activate response
type ActivateSessionResponse struct {
    Success bool   `json:"success"`
    Message string `json:"message,omitempty"`
}

// GET /workspaces response
type WorkspaceListResponse struct {
    Workspaces []Workspace `json:"workspaces"`
}
```

---

## 5. API Design

### Endpoint Specifications

#### `GET /sessions`

Returns all sessions sorted by `LastAccessedAt` in descending order (most recent first).

**Request:** None

**Response:** `SessionListResponse`
```json
{
  "sessions": [
    {
      "name": "utena-main",
      "workspace_path": "/Users/eleonora/dev/utena",
      "last_accessed_at": "2026-01-27T10:30:00Z",
      "is_active": true,
      "created_at": "2026-01-20T09:00:00Z"
    }
  ]
}
```

**Status Codes:**
- 200: Success
- 500: Internal server error

---

#### `POST /sessions`

Creates a new session with the provided name and workspace path.

**Request:** `CreateSessionRequest`
```json
{
  "name": "my-new-session",
  "workspace_path": "/Users/eleonora/dev/project"
}
```

**Response:** `CreateSessionResponse`
```json
{
  "session": {
    "name": "my-new-session",
    "workspace_path": "/Users/eleonora/dev/project",
    "last_accessed_at": "2026-01-27T10:35:00Z",
    "is_active": true,
    "created_at": "2026-01-27T10:35:00Z"
  }
}
```

**Status Codes:**
- 201: Session created successfully
- 400: Invalid request (missing fields, duplicate name, invalid workspace path)
- 500: Internal server error

**Side Effect:** Sends command to plugin via pipe to create and switch to the new session in Zellij.

---

#### `PUT /sessions/{name}/activate`

Marks the specified session as active and updates its `LastAccessedAt` timestamp.

**Path Parameter:** `name` - The session name

**Request:** None (empty body)

**Response:** `ActivateSessionResponse`
```json
{
  "success": true,
  "message": "Session activated successfully"
}
```

**Status Codes:**
- 200: Session activated successfully
- 404: Session not found
- 500: Internal server error

**Side Effect:** Sends command to plugin via pipe to switch to this session in Zellij.

---

#### `GET /workspaces`

Returns a list of available workspace directories discovered by the WorkspaceManager.

**Request:** None

**Response:** `WorkspaceListResponse`
```json
{
  "workspaces": [
    {
      "path": "/Users/eleonora/dev/utena",
      "name": "utena",
      "is_git_repo": true
    },
    {
      "path": "/Users/eleonora/projects/website",
      "name": "website",
      "is_git_repo": false
    }
  ]
}
```

**Status Codes:**
- 200: Success
- 500: Internal server error

---

#### `PUT /zellij/sessions` (existing, enhanced)

Receives session updates from the Zellij plugin. This endpoint is called when Zellij's session state changes.

**Request:** `UpdateSessionsRequest`
```json
{
  "id": "plugin-instance-id",
  "sessions": [
    {
      "name": "utena-main",
      "is_current_session": true
    },
    {
      "name": "old-project",
      "is_current_session": false
    }
  ]
}
```

**Response:**
```json
{
  "ok": "ok"
}
```

**Status Codes:**
- 200: Update processed successfully
- 400: Invalid request format
- 500: Internal server error

**Processing Logic:**
- Mark the session with `is_current_session: true` as active
- Update `LastAccessedAt` for the active session
- Create session entries for any unknown sessions
- Mark all other sessions as inactive

---

## 6. UI Specifications

### Session List UI

```
┌─ Session Picker ─────────────────────────┐
│                                           │
│  ❯ utena-main (~/dev/utena)              │
│    old-project (~/dev/old-project)       │
│    website (~/sites/mysite)              │
│                                           │
│  j/k: navigate  Enter: select  n: new    │
│  Esc/q: close                            │
└───────────────────────────────────────────┘
```

**Visual Elements:**
- Title bar: "Session Picker"
- Cursor indicator: `❯` symbol on selected line
- Session display: `{name} ({workspace_path})`
- Active session: Display in bold or with color indicator (optional enhancement)
- Footer: Keyboard hints for available actions
- Scrollable list if sessions exceed viewport

**Keyboard Actions:**
- `j` / `Down`: Move cursor down
- `k` / `Up`: Move cursor up
- `Enter`: Select current session and switch to it
- `n`: Toggle to New Session UI
- `Esc` / `q`: Close TUI without action

**Edge Cases:**
- Empty session list: Display "No sessions available. Press 'n' to create one."
- Single session: Still allow navigation, cursor on that item

---

### New Session UI - Workspace Selection

```
┌─ New Session ────────────────────────────┐
│ Select workspace:                         │
│                                           │
│  ❯ ~/dev/utena                           │
│    ~/dev/project-x                       │
│    ~/sites/blog                          │
│                                           │
│  j/k: navigate  Enter: select  Esc: back │
└───────────────────────────────────────────┘
```

**Visual Elements:**
- Title bar: "New Session"
- Instruction text: "Select workspace:"
- Cursor indicator: `❯` symbol on selected line
- Workspace display: Abbreviated path (use `~` for home directory)
- Footer: Keyboard hints

**Keyboard Actions:**
- `j` / `Down`: Move cursor down
- `k` / `Up`: Move cursor up
- `Enter`: Select workspace and advance to name input
- `Esc`: Return to Session List

**Edge Cases:**
- Empty workspace list: Display "No workspaces found. Configure root directories."
- Single workspace: Still allow navigation

---

### New Session UI - Name Input

```
┌─ New Session ────────────────────────────┐
│ Workspace: ~/dev/utena                    │
│                                           │
│ Session name (Enter for default):        │
│ utena-feature-branch█                    │
│                                           │
│  Enter: create  Esc: cancel              │
└───────────────────────────────────────────┘
```

**Visual Elements:**
- Title bar: "New Session"
- Selected workspace display: "Workspace: {path}"
- Prompt text: "Session name (Enter for default):"
- Text input field with cursor (█)
- Footer: Keyboard hints

**Keyboard Actions:**
- Alphanumeric keys: Type session name
- `Backspace`: Delete character
- `Enter`: Create session with provided name (or default if empty)
- `Esc`: Cancel and return to Session List

**Validation:**
- Reject names with spaces (show error message in footer)
- Reject duplicate session names (show error: "Session '{name}' already exists")
- Limit length to reasonable maximum (e.g., 50 characters)

**Default Name Generation:**
If user leaves input empty, generate default name using pattern:
- Format: `{workspace-name}-{counter}` or `{workspace-name}-{timestamp}`
- Example: `utena-1`, `utena-2`, or `utena-20260127`

---

## 7. Implementation Considerations

### State Management (Daemon)

**In-Memory Store:**
```go
type SessionStore struct {
    mu       sync.RWMutex
    sessions map[string]*Session
}
```

**Persistence:**
- Location: `~/.config/utena/sessions.json`
- Format: JSON array of Session objects
- Write on: session creation, activation, metadata updates
- Read on: daemon startup

**Operations:**
- `ListSessions() []Session`: Return all sessions sorted by MRU
- `GetSession(name string) (*Session, error)`: Retrieve specific session
- `CreateSession(name, path string) (*Session, error)`: Add new session
- `ActivateSession(name string) error`: Mark session active, update timestamp
- `UpdateFromZellij(updates []SessionUpdate) error`: Sync with Zellij state

---

### Bubbletea Patterns (TUI)

**Model Hierarchy:**
```go
type App struct {
    activeView tea.Model  // Current view (SessionList, NewSession, NameInput)
    err        error
}

type SessionListModel struct {
    sessions []Session
    cursor   int
    loading  bool
}

type NewSessionModel struct {
    workspaces []Workspace
    cursor     int
    loading    bool
}

type NameInputModel struct {
    workspace string
    input     textinput.Model
    err       string
}
```

**Message Types:**
```go
type sessionsLoadedMsg struct{ sessions []Session }
type workspacesLoadedMsg struct{ workspaces []Workspace }
type sessionActivatedMsg struct{}
type sessionCreatedMsg struct{ session Session }
type errorMsg struct{ err error }
type switchViewMsg struct{ view string }
```

**Command Functions:**
```go
func fetchSessions() tea.Msg
func fetchWorkspaces() tea.Msg
func activateSession(name string) tea.Msg
func createSession(name, path string) tea.Msg
```

---

### Plugin-Daemon Communication

**Plugin → Daemon (HTTP):**
- Pattern: Already established, continue using HTTP PUT for updates
- Endpoint: `PUT /zellij/sessions`
- Trigger: On Zellij `SessionUpdate` event
- Format: JSON payload with session list

**Daemon → Plugin (Pipe):**
- Pipe name: `utena-commands`
- Plugin subscribes to pipe in event loop
- Message format:
  ```json
  {
    "command": "switch_session",
    "session_name": "foo"
  }
  ```
- Plugin actions:
  - `switch_session`: Call `switch_session(name)` Zellij API
  - `create_session`: Call `new_tab_with_cwd(cwd)` or similar

**Implementation in Plugin:**
```rust
// In update() method, add pipe subscription
EventType::CustomMessage { payload, .. } => {
    let cmd: Command = serde_json::from_str(&payload)?;
    match cmd.command.as_str() {
        "switch_session" => switch_session(&cmd.session_name),
        "create_session" => create_session(&cmd.session_name, &cmd.workspace_path),
        _ => eprintln!("Unknown command: {}", cmd.command),
    }
}
```

---

### Error Handling

**TUI Error Display:**
- Show error messages in footer area (red text)
- Allow user to dismiss with any key or timeout
- Provide retry option for network failures

**Daemon Error Responses:**
- 400 Bad Request: Client error (validation failure, duplicate name)
- 404 Not Found: Session does not exist
- 500 Internal Server Error: Server-side failure
- Include descriptive error messages in response body

**Plugin Error Handling:**
- Log errors to Zellij debug output
- Fail gracefully (don't crash plugin)
- Show notification to user via Zellij toast (if API available)

---

### Keyboard Shortcuts Summary

| Key | Context | Action |
|-----|---------|--------|
| Ctrl+P | Plugin (any Zellij session) | Launch Session Picker TUI |
| j / Down | TUI (Session List, New Session) | Navigate down |
| k / Up | TUI (Session List, New Session) | Navigate up |
| Enter | TUI (Session List) | Select session and switch |
| Enter | TUI (New Session - workspace) | Select workspace, advance to name input |
| Enter | TUI (New Session - name input) | Create session and activate |
| n | TUI (Session List) | Toggle to New Session UI |
| Esc | TUI (any view) | Cancel/back/close |
| q | TUI (Session List) | Close TUI |

---

### Workspace Discovery

**Configuration:**
- Root directories specified in config file or environment variable
- Default: `~/.config/utena/config.json`
- Example config:
  ```json
  {
    "workspace_roots": [
      "~/dev",
      "~/projects",
      "~/sites"
    ]
  }
  ```

**Discovery Algorithm:**
1. Read configured root directories
2. Expand `~` to home directory
3. Scan each root directory for subdirectories (non-recursive)
4. Filter: include only directories, exclude hidden (optional)
5. Optionally detect git repositories (`is_git_repo` flag)
6. Sort alphabetically by name

**WorkspaceManager Configuration:**
```go
wm := workspace.NewManager(
    workspace.WithRootDir("/Users/eleonora/dev"),
    workspace.WithRootDir("/Users/eleonora/projects"),
)
```

---

### Session Naming

**Default Name Generation:**
- Option 1: `{workspace-name}-{counter}`
  - Example: `utena-1`, `utena-2`, `utena-3`
  - Counter increments for each new session in same workspace

- Option 2: `{workspace-name}-{timestamp}`
  - Example: `utena-20260127103000`
  - Timestamp format: YYYYMMDDHHMMSS

**Name Validation:**
- Allowed characters: alphanumeric, hyphens, underscores
- Disallowed: spaces, special characters
- Max length: 50 characters
- Case-sensitive (preserve user input)
- Uniqueness: enforce at creation time

**Duplicate Handling:**
- If duplicate detected, return 400 error
- Suggest alternative name: `{requested-name}-{counter}`
- Display error in TUI: "Session '{name}' already exists"

---

### Performance Considerations

**Session List Size:**
- Consider archiving old sessions after inactivity (e.g., 30 days)
- Limit active session count (e.g., max 100 sessions)
- Provide cleanup command or auto-prune feature

**Workspace Discovery:**
- Cache workspace list (refresh on demand or periodic interval)
- Lazy-load workspaces (only fetch when New Session UI opened)
- Limit depth of directory scanning (non-recursive)

**TUI Startup:**
- Pre-fetch session list on launch (use loading indicator)
- Async API calls to prevent blocking
- Fast initial render with "Loading..." state

**Daemon Efficiency:**
- Use read-write mutex for concurrent session access
- Batch disk writes (debounce persistence)
- Index sessions by name for O(1) lookups

---

## 8. Future Enhancements (Out of Scope)

**Session Templates/Layouts:**
- Save session layouts (tab/pane configurations)
- Restore layout when switching to session
- Template library for common workflows

**Recent Workspace Switching:**
- Track most-recently-used workspaces
- Quick switch to recent workspace without creating new session

**Session Tags/Categories:**
- Organize sessions by project, context, or tags
- Filter session list by category

**Search/Filter:**
- Real-time search in session list
- Filter by name, workspace, or tag
- Fuzzy matching for quick selection

**Delete/Archive Sessions:**
- Remove sessions from TUI interface
- Archive instead of delete (soft delete)
- Bulk operations (delete multiple sessions)

**Session Statistics:**
- Track session usage time
- Display last accessed date/time
- Most active sessions report

**Configuration UI:**
- Manage workspace root directories from TUI
- Configure keyboard shortcuts
- Customize UI colors/themes

---

## 9. Testing Strategy

**Unit Tests:**
- Session store operations (create, activate, list, update)
- Workspace discovery logic
- Session name validation and generation
- MRU sorting algorithm

**Integration Tests:**
- API endpoint behavior (request/response validation)
- TUI keyboard navigation and view transitions
- Plugin-daemon communication (mock HTTP and pipe)

**Manual Testing Scenarios:**
1. Launch TUI with Ctrl+P, navigate sessions, select one
2. Create new session with custom name
3. Create new session with default name
4. Switch between sessions, verify MRU order updates
5. Test with empty session list
6. Test with single session
7. Test workspace discovery with multiple roots
8. Test error handling (daemon unreachable, invalid input)
9. Test TUI dismissal (Esc/q in various states)
10. Test duplicate session name rejection

**End-to-End Testing:**
- Full workflow: Launch TUI → Select session → Verify Zellij switches
- Full workflow: Launch TUI → Create session → Verify session appears in Zellij
- Multiple concurrent TUI instances (edge case)
- Daemon restart with persisted sessions

---

## 10. Open Questions

1. **Session Cleanup Policy**: Should old sessions be automatically archived or deleted? What criteria (inactivity period, max count)?

2. **Workspace Configuration**: Should workspace roots be configurable via TUI or only via config file?

3. **Visual Indicators**: Should the active session be prominently highlighted (bold, color, icon)?

4. **Error Recovery**: If daemon is unreachable, should TUI retry automatically or just show error?

5. **Keyboard Shortcuts**: Is Ctrl+P the best choice, or should it be configurable? Any conflicts with Zellij defaults?

6. **Session Metadata**: Should we track additional data like creation time, last modified, total usage time?

7. **Multi-User Support**: Should sessions be per-user or shared? Implications for persistence location.

---

## 11. Success Criteria

The Session Picker feature will be considered complete when:

1. ✅ User can trigger TUI launch with Ctrl+P from any Zellij session
2. ✅ Session List displays all sessions in MRU order
3. ✅ User can navigate sessions with j/k and select with Enter
4. ✅ Selecting a session switches to it in Zellij and dismisses TUI
5. ✅ User can toggle to New Session UI with 'n' key
6. ✅ New Session UI displays available workspaces
7. ✅ User can select workspace and provide session name
8. ✅ Default session name is generated when input is empty
9. ✅ New session is created and activated in Zellij
10. ✅ Sessions persist across daemon restarts
11. ✅ All API endpoints function correctly (unit/integration tests pass)
12. ✅ Error handling provides clear feedback to users
13. ✅ Documentation is complete and accurate

---

## 12. Implementation Timeline

**Phase 1: Daemon Foundation (Week 1)**
- Session store with persistence
- API endpoints implementation
- Workspace discovery enhancement

**Phase 2: TUI Development (Week 2)**
- Session List UI with navigation
- New Session UI with workspace picker
- Name input screen with validation
- API client integration

**Phase 3: Plugin Integration (Week 3)**
- Ctrl+P keyboard shortcut handler
- TUI pane launching
- Pipe subscription for daemon commands
- Session switching via Zellij API

**Phase 4: Testing & Polish (Week 4)**
- End-to-end testing
- Bug fixes and edge case handling
- Documentation updates
- Performance optimization

**Note:** Timeline is illustrative; actual duration depends on team size and priorities.
