# Event Bus Pattern

## Overview

The event bus provides **unidirectional** communication between modules. Use it when Module A needs to notify Module B of something, but Module B should not directly depend on Module A.

## When to Use

- **Recommended**: When a lower-level module needs to notify a higher-level module
- **Not recommended**: When modules can have direct dependencies without creating cycles

## Architecture

```
Module A (publishes events) → EventBus → Module B (subscribes)
Module B → (direct calls) → Module A
```

**Example**: Session module publishes events, Zellij module subscribes to them. Zellij directly calls SessionService methods.

See: `internal/api/daemon.go:24-38`

## Implementation

### 1. Define Events

Create event type constants and data structures.

**Required**: Event names should follow `module.action` naming (e.g., `session.create_requested`)

See: `internal/eventbus/events.go`

### 2. Publisher Side

**Required**: Publish events from the **service layer**, not controllers.

```go
// In service method after business logic
event := eventbus.Event{
    Type: eventbus.SessionCreateRequested,
    Data: eventbus.SessionCreateRequestedEvent{...},
}
s.eventBus.Publish(ctx, event)
```

See: `internal/session/sessionservice.go:64-71`

### 3. Subscriber Side

**Required**: Subscribe to events in the `OnAppStart()` lifecycle method.

```go
func (z *ZellijService) OnAppStart(ctx context.Context) error {
    z.eventBus.Subscribe(eventbus.SessionCreateRequested, z.handleEvent)
    return nil
}
```

See: `internal/zellij/zellijservice.go:17-20`

### 4. Module Wiring

Pass the event bus to modules during construction:

See: `internal/api/daemon.go:24-28`

## Rules

1. **Events flow in one direction only** - If Module B needs to call Module A, use direct dependencies, not events
2. **Service layer owns events** - Controllers should not publish or subscribe to events
3. **Subscribe in OnAppStart** - Event subscriptions are part of module initialization
4. **No circular event flows** - If A publishes to B and B publishes to A, refactor to use direct calls instead
