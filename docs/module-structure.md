# Module Structure

## Overview

Modules encapsulate domain logic and follow a consistent structure with clear lifecycle hooks.

## Module Components

### Required Components

Each module should have:

1. **Store** - Data persistence layer
2. **Service** - Business logic layer
3. **Controller** - HTTP request/response handling
4. **Router** - Route definitions
5. **Module** - Composition and lifecycle management

See example: `internal/session/sessionmodule.go`

## Layers and Responsibilities

### Store Layer

- Data access and persistence
- No business logic
- No external dependencies beyond data structures

See: `internal/session/sessionstore.go`

### Service Layer

- **Required**: All business logic lives here
- **Required**: Event publishing happens here
- Can depend on other services directly
- Can depend on event bus for publishing
- Should not know about HTTP (no request/response types)

See: `internal/session/sessionservice.go`

### Controller Layer

- HTTP request parsing and validation
- Response formatting
- **Required**: Should be thin - delegate to service layer
- **Not allowed**: No event bus dependencies
- **Not allowed**: No business logic

See: `internal/session/sessioncontroller.go`

### Router Layer

- Route definitions only
- Maps HTTP paths to controller methods

See: `internal/session/sessionrouter.go`

## Lifecycle Hooks

### Required Methods

All modules must implement:

```go
OnAppStart(ctx context.Context) error  // Initialization
OnAppEnd(ctx context.Context) error    // Cleanup
Routes() chi.Router                     // HTTP routes
```

### OnAppStart Usage

**Required**: Use for event subscriptions

See: `internal/zellij/zellijservice.go:17-20`

**Required**: Call on all sub-components in order (Store → Service → etc.)

See: `internal/session/sessionmodule.go:31-41`

## Module Dependencies

### Dependency Rules

1. **Recommended**: Keep dependencies unidirectional
2. **Required**: Use event bus when direct dependency would create a cycle
3. **Required**: Pass dependencies through constructors, not global variables

### Wiring in Daemon

Modules are constructed and initialized in `internal/api/daemon.go`:

1. Create event bus
2. Construct modules (pass dependencies)
3. Call OnAppStart on all modules (in dependency order)
4. Start HTTP server
5. Call OnAppEnd on shutdown (in reverse order)

See: `internal/api/daemon.go:19-56`

## Example Module

For a complete example, see the session module:
- Module: `internal/session/sessionmodule.go`
- Service: `internal/session/sessionservice.go`
- Controller: `internal/session/sessioncontroller.go`
- Router: `internal/session/sessionrouter.go`
- Store: `internal/session/sessionstore.go`
