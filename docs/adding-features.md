# Adding Features

## Adding a New HTTP Endpoint

### 1. Add Service Method

**Required**: Business logic goes in service layer.

```go
// internal/session/sessionservice.go
func (s *SessionService) DoSomething(ctx context.Context, id string) error {
    // Business logic here

    // If other modules need to know, publish event
    event := eventbus.Event{Type: "session.something_happened", Data: ...}
    s.eventBus.Publish(ctx, event)

    return nil
}
```

### 2. Add Controller Method

**Required**: Keep controllers thin - just HTTP concerns.

```go
// internal/session/sessioncontroller.go
func (c *SessionController) DoSomething(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    id := chi.URLParam(r, "id")

    if err := c.service.DoSomething(ctx, id); err != nil {
        render.Render(w, r, common.ErrUnknown(err))
        return
    }

    render.NoContent(w, r)
}
```

### 3. Add Route

```go
// internal/session/sessionrouter.go
r.Post("/{id}/action", sr.controller.DoSomething)
```

## Adding Event-Based Communication

### 1. Define Event

```go
// internal/eventbus/events.go
const ThingHappened = "module.thing_happened"

type ThingHappenedEvent struct {
    ID   string
    Data string
}
```

### 2. Publish Event (in Service)

See: `internal/session/sessionservice.go:64-71`

### 3. Subscribe to Event (in OnAppStart)

```go
// internal/othermodule/service.go
func (s *Service) OnAppStart(ctx context.Context) error {
    s.eventBus.Subscribe(eventbus.ThingHappened, s.handleThingHappened)
    return nil
}

func (s *Service) handleThingHappened(ctx context.Context, event eventbus.Event) error {
    data, ok := event.Data.(eventbus.ThingHappenedEvent)
    if !ok {
        return nil
    }
    // Handle event
    return nil
}
```

## Adding a New Module

### 1. Create Module Structure

Create files following the pattern in `internal/session/`:
- `types.go` - Data structures
- `store.go` - Data persistence
- `service.go` - Business logic
- `controller.go` - HTTP handlers
- `router.go` - Route definitions
- `module.go` - Composition

### 2. Implement Lifecycle

**Required**: Implement these in module.go:

```go
func (m *Module) OnAppStart(ctx context.Context) error
func (m *Module) OnAppEnd(ctx context.Context) error
func (m *Module) Routes() chi.Router
```

### 3. Wire in Daemon

Add to `internal/api/daemon.go`:

```go
newModule := newmodule.NewModule(dependencies, bus)

// In OnAppStart section
if err := newModule.OnAppStart(ctx); err != nil {
    log.Fatalf("Failed to initialize module: %v", err)
}

// Mount routes
r.Mount("/path", newModule.Routes())

// In OnAppEnd section (reverse order)
if err := newModule.OnAppEnd(ctx); err != nil {
    log.Printf("Error cleaning up module: %v", err)
}
```

See: `internal/api/daemon.go:24-56`

## Testing New Features

### Service Tests

Test business logic with mocked dependencies.

See: `internal/session/sessionservice_test.go`

### Controller Tests

Use httptest to test HTTP handling.

See: `internal/session/sessionrouter_test.go`

### Integration Tests

Test full module stack with real HTTP requests.

See: `internal/api/daemon_test.go`
