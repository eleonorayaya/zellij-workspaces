# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Utena is a workspace management system for Zellij, consisting of three interconnected components:
- **daemon**: HTTP API server (Go) that manages workspace state and Zellij session information
- **tui**: Terminal UI client (Go + Bubbletea) for interacting with the daemon
- **zellij-plugin**: WebAssembly plugin (Rust) that runs inside Zellij and communicates with the daemon

The architecture enables the Zellij plugin to detect session changes and push updates to the daemon via HTTP, while the TUI can query the daemon to display workspace information.

## Build & Run Commands

This project uses [Task](https://taskfile.dev) for builds. All commands use `task <target>`.

### Go Components (daemon & tui)

Build and run the daemon (HTTP server on port 3333):
```bash
task daemon:build
task daemon:run
```

Build and run the TUI (uses BUBBLETEA_LOG=tui.log for logging):
```bash
task tui:build
task tui:run
```

Format Go code:
```bash
task fmt
```

### Zellij Plugin (Rust/WASM)

The plugin is in the `zellij-plugin/` subdirectory with its own Taskfile.

Build the plugin (compiles to wasm32-wasip1):
```bash
cd zellij-plugin
task build
```

Build and deploy to Zellij (copies to ~/.config/zellij/plugins/ and reloads):
```bash
cd zellij-plugin
task deploy
```

Watch Zellij logs:
```bash
cd zellij-plugin
task logs
```

### Development Workflow

Open the full development environment (requires Zellij):
```bash
task dev
```

This creates a Zellij layout (defined in `dev.kdl`) with panes for:
- Running the daemon server
- Tailing Zellij logs
- Auto-recompiling and reloading the plugin
- The active plugin instance

## Architecture

### Component Communication

1. **Zellij Plugin → Daemon**: The plugin subscribes to Zellij's `SessionUpdate` events and sends HTTP PUT requests to `http://localhost:3333/zellij/sessions` with session state (zellij-plugin/src/main.rs:147-169)

2. **TUI → Daemon**: The TUI makes HTTP GET requests to `http://localhost:3333/sessions` to fetch workspace/session data (internal/tui/app.go:56-72)

3. **Daemon API**: Uses chi router, serves on port 3333, mounts two controllers:
   - `/sessions` - session management endpoints
   - `/zellij` - Zellij-specific endpoints (internal/api/daemon.go:41-42)

### Go Module Structure

- `cmd/daemon/main.go` - daemon entry point
- `cmd/tui/main.go` - TUI entry point
- `internal/api/` - HTTP API and daemon server
- `internal/session/` - session controller logic
- `internal/workspace/` - workspace discovery and management
- `internal/zellij/` - Zellij service and controller
- `internal/tui/` - Bubbletea TUI application
- `internal/common/` - shared utilities

### Key Patterns

**Workspace Manager** (internal/workspace/workspace.go): Uses functional options pattern (`WithRootDir()`) to configure root directories for workspace discovery. Scans directories to find workspace folders.

**Zellij Service** (internal/zellij/zellijservice.go): Maintains session state and provides `OnSessionUpdate()` hook for handling session changes from the plugin.

**Plugin Event Loop** (zellij-plugin/src/main.rs): The plugin subscribes to multiple Zellij event types (Key, SessionUpdate, WebRequestResult, etc.) and handles them in the `update()` method. Uses `web_request()` to communicate with the daemon.

## Dependencies

- **Go**: chi (HTTP router), bubbletea (TUI framework)
- **Rust**: zellij-tile 0.42.2 (plugin API), serde/serde_json (serialization)
- **External**: Requires Zellij terminal multiplexer to be installed

## Testing

Currently no test infrastructure is set up. When adding tests, follow Go convention of `*_test.go` files alongside source files.

## Code Style

### Comments

Do not add comments to code unless explicitly requested by the user. This includes:
- Explanatory comments describing what code does
- Comments documenting functions, methods, or types
- Comments explaining implementation details
- TODOs or FIXMEs (unless specifically asked)

The code should be self-documenting through clear naming and structure. Only add comments when the user explicitly asks for them.
