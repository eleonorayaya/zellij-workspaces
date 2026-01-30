# Architecture Overview

## System Components

Utena consists of three main components:
- **daemon** - HTTP API server managing workspace and session state
- **tui** - Terminal UI client for user interaction
- **zellij-plugin** - WebAssembly plugin running inside Zellij

## Daemon Architecture

### Module Dependencies

```
workspace (no dependencies)
    ↓
session (depends on: workspace, eventbus)
    ↓
zellij (depends on: session, eventbus)
```

**Key principle**: Dependencies flow downward. Lower modules never depend on higher modules directly.

### Event Flow

```
session → (events) → eventbus → (events) → zellij
zellij → (direct calls) → session
```

**Rationale**:
- Session doesn't know about Zellij (no dependency)
- Zellij needs to update session state (direct calls)
- Session needs to notify Zellij of user actions (events)

See: `docs/event-bus.md` for details

## Communication Patterns

### Plugin → Daemon (Zellij Updates)

HTTP PUT `/zellij/sessions` with current session state from plugin.

Flow:
1. Plugin detects session changes
2. Sends HTTP request to daemon
3. ZellijController receives request
4. ZellijService calls SessionService methods directly
5. Session state updated

See: `internal/zellij/zellijservice.go:28-71`

### Daemon → Plugin (User Actions)

HTTP API triggers Zellij plugin commands via named pipes.

Flow:
1. HTTP POST `/sessions` creates new session
2. SessionService publishes `SessionCreateRequested` event
3. ZellijService subscribed to event
4. ZellijService sends command to plugin via pipe
5. Plugin executes command

See:
- Service publishing: `internal/session/sessionservice.go:64-71`
- Zellij subscribing: `internal/zellij/zellijservice.go:17-20`
- Command sending: `internal/zellij/zellijservice.go:78-89`

### TUI → Daemon

HTTP GET requests to fetch session/workspace data.

Flow:
1. TUI polls `/sessions` endpoint
2. SessionController returns current state
3. TUI renders in terminal

See: `internal/tui/app.go:56-72`

## Dependency Inversion

When Module A needs Module B's functionality but direct dependency would create a cycle:

1. **Preferred**: Use event bus for one direction
2. **Alternative**: Extract shared interface to common package

Example: Session and Zellij modules would create a cycle if Session depended on Zellij. Instead, Session publishes events that Zellij subscribes to.

## Testing

Each layer can be tested independently:

- **Stores**: Test with in-memory data
- **Services**: Mock store dependencies
- **Controllers**: Use httptest with real service
- **Integration**: Test full module stack

See test files adjacent to source files (e.g., `internal/session/sessionservice_test.go`)
