# Plan: Session Picker API Implementation

## Overview
Implement the backend APIs for the Session Picker feature as specified in `docs/session-picker-spec.md`. This includes creating session and workspace management services with lifecycle interfaces, four API endpoints, and comprehensive test coverage.

## User Requirements

### Adjustments from Specification:
1. **Session references workspace UUID** instead of path for durability
2. **Session struct** lives in `internal/session/session.go` (separate file)
3. **Generic lifecycle interface** for services with `OnAppStart/OnAppEnd` methods
4. **In-memory storage only** - no file persistence for now
5. **SessionController** moved to `internal/api/` directory
6. **Replace WorkspaceManager** with WorkspaceService/WorkspaceStore following same patterns
7. **Hard-code workspaces** for now (skip discovery mechanism)
8. **Comprehensive test coverage** for all new functionality
9. **Standardized module pattern** - each module exports `New<Name>Module()` function that returns struct with router, services, stores, and implements lifecycle interface

## Current State
Based on codebase exploration:
- **SessionController**: Stub with single GET / endpoint returning "sessions"
- **ZellijController**: Has `PUT /zellij/sessions` endpoint that currently doesn't process updates
- **WorkspaceManager**: Has `ListWorkspaces()` but returns only path field
- **Architecture**: Controllers are chi.Router factories, services hold state, dependency injection via constructors

## API Requirements

### Endpoints
1. `GET /sessions` - List all sessions sorted by MRU (most recently used)
2. `POST /sessions` - Create new session with name and workspace UUID
3. `PUT /sessions/{name}/activate` - Activate existing session, update last accessed time
4. `GET /workspaces` - List available workspaces
5. `PUT /zellij/sessions` (enhance) - Process session updates from Zellij plugin

### Data Structures

**Workspace** (with UUID):
```go
type Workspace struct {
    ID        string `json:"id"`         // UUID
    Name      string `json:"name"`       // Display name
    Path      string `json:"path"`       // Filesystem path
    IsGitRepo bool   `json:"is_git_repo"`
}
```

**Session** (references workspace by UUID):
```go
type Session struct {
    Name           string    `json:"name"`
    WorkspaceID    string    `json:"workspace_id"`    // References Workspace.ID
    LastAccessedAt time.Time `json:"last_accessed_at"`
    IsActive       bool      `json:"is_active"`
    CreatedAt      time.Time `json:"created_at"`
}
```

## Module Pattern

### Overview
Each module (workspace, session, etc.) follows a standardized pattern:
- **Module file**: `<name>module.go` exports `New<Name>Module()` function
- **Module struct**: Contains router (if applicable), services, stores
- **Lifecycle**: Module implements `OnAppStart/OnAppEnd` interface
- **Initialization**: Daemon instantiates modules in dependency order, then calls `OnAppStart` on each

### Example Module Structure
```go
// workspacecontroller.go - HTTP handlers
type WorkspaceController struct {
    service *WorkspaceService
}

func NewWorkspaceController(service *WorkspaceService) *WorkspaceController {
    return &WorkspaceController{service: service}
}

func (c *WorkspaceController) ListWorkspaces(w http.ResponseWriter, r *http.Request) {
    // Handler implementation
}

// workspacerouter.go - Router setup
func NewWorkspaceRouter(service *WorkspaceService) chi.Router {
    controller := NewWorkspaceController(service)
    r := chi.NewRouter()
    r.Get("/", controller.ListWorkspaces)
    return r
}

// workspacemodule.go - Module bundle
type WorkspaceModule struct {
    Store   *WorkspaceStore
    Service *WorkspaceService
    Router  chi.Router
}

func NewWorkspaceModule() *WorkspaceModule {
    store := NewWorkspaceStore()
    service := NewWorkspaceService(store)
    router := NewWorkspaceRouter(service)

    return &WorkspaceModule{
        Store:   store,
        Service: service,
        Router:  router,
    }
}

func (m *WorkspaceModule) OnAppStart(ctx context.Context) error {
    return m.Service.OnAppStart(ctx)
}

func (m *WorkspaceModule) OnAppEnd(ctx context.Context) error {
    return m.Service.OnAppEnd(ctx)
}
```

## Implementation Strategy

### Phase 0: Foundation

#### 0.1 Create Lifecycle Interface
**File**: `internal/common/lifecycle.go` (new)

Define interface for modules with startup/shutdown hooks:
```go
package common

import "context"

// Module defines the lifecycle interface for modules
type Module interface {
    OnAppStart(ctx context.Context) error
    OnAppEnd(ctx context.Context) error
}
```

This interface allows daemon to iterate through all modules and call lifecycle methods on app startup and shutdown.

### Phase 1: Workspace Module

#### 1.1 Create Workspace Entity
**File**: `internal/workspace/workspace.go` (replace existing)

Define Workspace struct:
```go
package workspace

type Workspace struct {
    ID        string `json:"id"`         // Simple ID like "ws-1", "ws-2"
    Name      string `json:"name"`       // Display name
    Path      string `json:"path"`       // Filesystem path
    IsGitRepo bool   `json:"is_git_repo"` // Git repository flag
}
```

#### 1.2 Create WorkspaceStore
**File**: `internal/workspace/workspacestore.go` (new)

In-memory store for workspaces:
```go
package workspace

import "sync"

type WorkspaceStore struct {
    mu         sync.RWMutex
    workspaces map[string]*Workspace  // indexed by ID
}

func NewWorkspaceStore() *WorkspaceStore
func (s *WorkspaceStore) GetByID(id string) (*Workspace, error)
func (s *WorkspaceStore) GetByPath(path string) (*Workspace, error)
func (s *WorkspaceStore) List() []Workspace
func (s *WorkspaceStore) Add(ws *Workspace) error
```

#### 1.3 Create WorkspaceService
**File**: `internal/workspace/workspaceservice.go` (new)

Service managing workspace business logic:
```go
package workspace

import "context"

type WorkspaceService struct {
    store *WorkspaceStore
}

func NewWorkspaceService(store *WorkspaceStore) *WorkspaceService

// OnAppStart initializes hard-coded workspaces
func (s *WorkspaceService) OnAppStart(ctx context.Context) error

// OnAppEnd cleans up (no-op for now)
func (s *WorkspaceService) OnAppEnd(ctx context.Context) error

// Business logic
func (s *WorkspaceService) ListWorkspaces(ctx context.Context) ([]Workspace, error)
func (s *WorkspaceService) GetWorkspace(ctx context.Context, id string) (*Workspace, error)
func (s *WorkspaceService) GetWorkspaceByPath(ctx context.Context, path string) (*Workspace, error)
```

**OnAppStart** implementation:
```go
func (s *WorkspaceService) OnAppStart(ctx context.Context) error {
    workspaces := []*Workspace{
        {
            ID:        "ws-1",
            Name:      "utena",
            Path:      "/Users/eleonora/dev/utena",
            IsGitRepo: true,
        },
        {
            ID:        "ws-2",
            Name:      "example-project",
            Path:      "/Users/eleonora/dev/example",
            IsGitRepo: false,
        },
    }

    for _, ws := range workspaces {
        s.store.Add(ws)
    }
    return nil
}
```

#### 1.4 Create Workspace Controller
**File**: `internal/workspace/workspacecontroller.go` (new)

HTTP handlers for workspace endpoints:
```go
package workspace

import (
    "github.com/go-chi/render"
    "net/http"
    "github.com/eleonorayaya/utena/internal/common"
)

type WorkspaceController struct {
    service *WorkspaceService
}

func NewWorkspaceController(service *WorkspaceService) *WorkspaceController {
    return &WorkspaceController{service: service}
}

func (c *WorkspaceController) ListWorkspaces(w http.ResponseWriter, r *http.Request) {
    workspaces, err := c.service.ListWorkspaces(r.Context())
    if err != nil {
        render.Render(w, r, common.ErrUnknown(err))
        return
    }
    render.Render(w, r, &WorkspaceListResponse{Workspaces: workspaces})
}
```

#### 1.5 Create Workspace Router
**File**: `internal/workspace/workspacerouter.go` (new)

Router setup that wires controller to routes:
```go
package workspace

import "github.com/go-chi/chi/v5"

func NewWorkspaceRouter(service *WorkspaceService) chi.Router {
    controller := NewWorkspaceController(service)
    r := chi.NewRouter()
    r.Get("/", controller.ListWorkspaces)
    return r
}
```

#### 1.6 Create Workspace API Types
**File**: `internal/workspace/types.go` (new)

Request/response types:
```go
package workspace

import "net/http"

type WorkspaceListResponse struct {
    Workspaces []Workspace `json:"workspaces"`
}

func (r *WorkspaceListResponse) Render(w http.ResponseWriter, req *http.Request) error {
    return nil
}
```

#### 1.7 Create Workspace Module
**File**: `internal/workspace/workspacemodule.go` (new)

Module that bundles all workspace components:
```go
package workspace

import (
    "context"
    "github.com/go-chi/chi/v5"
)

// WorkspaceModule bundles all workspace-related components
type WorkspaceModule struct {
    Store   *WorkspaceStore
    Service *WorkspaceService
    Router  chi.Router
}

// NewWorkspaceModule initializes the workspace module
func NewWorkspaceModule() *WorkspaceModule {
    store := NewWorkspaceStore()
    service := NewWorkspaceService(store)
    router := NewWorkspaceRouter(service)  // Uses public function from router.go

    return &WorkspaceModule{
        Store:   store,
        Service: service,
        Router:  router,
    }
}

// OnAppStart initializes module state
func (m *WorkspaceModule) OnAppStart(ctx context.Context) error {
    return m.Service.OnAppStart(ctx)
}

// OnAppEnd cleans up module resources
func (m *WorkspaceModule) OnAppEnd(ctx context.Context) error {
    return m.Service.OnAppEnd(ctx)
}
```

### Phase 2: Session Module

#### 2.1 Create Session Entity
**File**: `internal/session/session.go` (new)

Define Session struct:
```go
package session

import "time"

type Session struct {
    Name           string    `json:"name"`
    WorkspaceID    string    `json:"workspace_id"`    // References Workspace.ID
    LastAccessedAt time.Time `json:"last_accessed_at"`
    IsActive       bool      `json:"is_active"`
    CreatedAt      time.Time `json:"created_at"`
}
```

#### 2.2 Create SessionStore
**File**: `internal/session/sessionstore.go` (new)

In-memory store for sessions:
```go
package session

import "sync"

type SessionStore struct {
    mu       sync.RWMutex
    sessions map[string]*Session  // indexed by session name
}

func NewSessionStore() *SessionStore
func (s *SessionStore) Get(name string) (*Session, error)
func (s *SessionStore) List() []Session
func (s *SessionStore) Add(session *Session) error
func (s *SessionStore) Update(session *Session) error
func (s *SessionStore) Delete(name string) error
```

#### 2.3 Create SessionService
**File**: `internal/session/sessionservice.go` (new)

Service managing session business logic:
```go
package session

import (
    "context"
    "github.com/eleonorayaya/utena/internal/workspace"
)

type SessionService struct {
    store            *SessionStore
    workspaceService *workspace.WorkspaceService  // To validate workspace IDs
}

func NewSessionService(store *SessionStore, workspaceService *workspace.WorkspaceService) *SessionService

// Lifecycle methods
func (s *SessionService) OnAppStart(ctx context.Context) error  // No-op for now
func (s *SessionService) OnAppEnd(ctx context.Context) error    // No-op (no persistence)

// Business logic
func (s *SessionService) ListSessions(ctx context.Context) ([]Session, error)
func (s *SessionService) GetSession(ctx context.Context, name string) (*Session, error)
func (s *SessionService) CreateSession(ctx context.Context, name, workspaceID string) (*Session, error)
func (s *SessionService) ActivateSession(ctx context.Context, name string) error
func (s *SessionService) UpdateFromZellij(ctx context.Context, updates []SessionUpdate) error
```

**Implementation details:**
- `ListSessions`: Sort by LastAccessedAt descending (MRU)
- `CreateSession`: Validate workspace ID exists, validate session name, create with timestamps
- `ActivateSession`: Mark all sessions inactive, then mark target as active with updated LastAccessedAt
- `UpdateFromZellij`: Sync sessions from Zellij plugin updates

#### 2.4 Create Validation Logic
**File**: `internal/session/validation.go` (new)

Validation functions:
```go
package session

func validateSessionName(name string) error
func generateDefaultSessionName(workspaceName string, existingNames map[string]bool) string
```

**Validation rules:**
- Alphanumeric + hyphens/underscores only
- Max 50 characters
- No empty names
- No duplicates

**Default name generation:**
- Try `{workspace-name}` first
- If exists, try `{workspace-name}-{counter}` (1-99)
- Fallback to `{workspace-name}-{timestamp}`

#### 2.5 Create Session Controller
**File**: `internal/session/sessioncontroller.go` (new)

HTTP handlers for session endpoints:
```go
package session

import (
    "github.com/go-chi/chi/v5"
    "github.com/go-chi/render"
    "net/http"
    "github.com/eleonorayaya/utena/internal/common"
    "github.com/eleonorayaya/utena/internal/workspace"
)

type SessionController struct {
    sessionService   *SessionService
    workspaceService *workspace.WorkspaceService
}

func NewSessionController(sessionService *SessionService, workspaceService *workspace.WorkspaceService) *SessionController {
    return &SessionController{
        sessionService:   sessionService,
        workspaceService: workspaceService,
    }
}

func (c *SessionController) ListSessions(w http.ResponseWriter, r *http.Request) {
    sessions, err := c.sessionService.ListSessions(r.Context())
    if err != nil {
        render.Render(w, r, common.ErrUnknown(err))
        return
    }
    render.Render(w, r, &SessionListResponse{Sessions: sessions})
}

func (c *SessionController) CreateSession(w http.ResponseWriter, r *http.Request) {
    req := &CreateSessionRequest{}
    if err := render.Bind(r, req); err != nil {
        render.Render(w, r, common.ErrInvalidRequest(err))
        return
    }

    // Generate default name if empty
    name := req.Name
    if name == "" {
        workspace, err := c.workspaceService.GetWorkspace(r.Context(), req.WorkspaceID)
        if err != nil {
            render.Render(w, r, common.ErrInvalidRequest(err))
            return
        }
        sessions, _ := c.sessionService.ListSessions(r.Context())
        existingNames := make(map[string]bool)
        for _, s := range sessions {
            existingNames[s.Name] = true
        }
        name = generateDefaultSessionName(workspace.Name, existingNames)
    }

    session, err := c.sessionService.CreateSession(r.Context(), name, req.WorkspaceID)
    if err != nil {
        // Check error type for appropriate status code
        if strings.Contains(err.Error(), "already exists") ||
           strings.Contains(err.Error(), "invalid") {
            render.Render(w, r, common.ErrInvalidRequest(err))
        } else {
            render.Render(w, r, common.ErrUnknown(err))
        }
        return
    }

    render.Status(r, http.StatusCreated)
    render.Render(w, r, &CreateSessionResponse{Session: *session})
}

func (c *SessionController) ActivateSession(w http.ResponseWriter, r *http.Request) {
    name := chi.URLParam(r, "name")
    if name == "" {
        render.Render(w, r, common.ErrInvalidRequest(errors.New("session name required")))
        return
    }

    err := c.sessionService.ActivateSession(r.Context(), name)
    if err != nil {
        if strings.Contains(err.Error(), "not found") {
            render.Render(w, r, common.ErrNotFound())
        } else {
            render.Render(w, r, common.ErrUnknown(err))
        }
        return
    }

    render.Render(w, r, &ActivateSessionResponse{
        Success: true,
        Message: "Session activated successfully",
    })
}
```

#### 2.6 Create Session Router
**File**: `internal/session/sessionrouter.go` (new)

Router setup that wires controller to routes:
```go
package session

import (
    "github.com/go-chi/chi/v5"
    "github.com/eleonorayaya/utena/internal/workspace"
)

func NewSessionRouter(sessionService *SessionService, workspaceService *workspace.WorkspaceService) chi.Router {
    controller := NewSessionController(sessionService, workspaceService)
    r := chi.NewRouter()
    r.Get("/", controller.ListSessions)
    r.Post("/", controller.CreateSession)
    r.Put("/{name}/activate", controller.ActivateSession)
    return r
}
```

#### 2.7 Create Session API Types
**File**: `internal/session/types.go` (new)

Request/response types:
```go
package session

import "net/http"

// SessionUpdate - from Zellij plugin
type SessionUpdate struct {
    Name             string `json:"name"`
    IsCurrentSession bool   `json:"is_current_session"`
}

// SessionListResponse - GET /sessions response
type SessionListResponse struct {
    Sessions []Session `json:"sessions"`
}

// CreateSessionRequest - POST /sessions request
type CreateSessionRequest struct {
    Name        string `json:"name"`         // Optional, generate default if empty
    WorkspaceID string `json:"workspace_id"` // Required
}

// CreateSessionResponse - POST /sessions response
type CreateSessionResponse struct {
    Session Session `json:"session"`
}

// ActivateSessionResponse - PUT /sessions/{name}/activate response
type ActivateSessionResponse struct {
    Success bool   `json:"success"`
    Message string `json:"message,omitempty"`
}

// Implement Bind() for requests
func (r *CreateSessionRequest) Bind(req *http.Request) error {
    if r.WorkspaceID == "" {
        return errors.New("workspace_id is required")
    }
    return nil
}

// Implement Render() for responses
func (r *SessionListResponse) Render(w http.ResponseWriter, req *http.Request) error {
    return nil
}

func (r *CreateSessionResponse) Render(w http.ResponseWriter, req *http.Request) error {
    return nil
}

func (r *ActivateSessionResponse) Render(w http.ResponseWriter, req *http.Request) error {
    return nil
}
```

#### 2.8 Create Session Module
**File**: `internal/session/sessionmodule.go` (new)

Module that bundles all session components:
```go
package session

import (
    "context"
    "github.com/go-chi/chi/v5"
    "github.com/eleonorayaya/utena/internal/workspace"
)

// SessionModule bundles all session-related components
type SessionModule struct {
    Store   *SessionStore
    Service *SessionService
    Router  chi.Router
}

// NewSessionModule initializes the session module with workspace dependency
func NewSessionModule(workspaceModule *workspace.WorkspaceModule) *SessionModule {
    store := NewSessionStore()
    service := NewSessionService(store, workspaceModule.Service)
    router := NewSessionRouter(service, workspaceModule.Service)  // Uses public function from router.go

    return &SessionModule{
        Store:   store,
        Service: service,
        Router:  router,
    }
}

// OnAppStart initializes module state
func (m *SessionModule) OnAppStart(ctx context.Context) error {
    return m.Service.OnAppStart(ctx)
}

// OnAppEnd cleans up module resources
func (m *SessionModule) OnAppEnd(ctx context.Context) error {
    return m.Service.OnAppEnd(ctx)
}
```

### Phase 3: Zellij Integration

#### 3.1 Enhance ZellijController
**File**: `internal/zellij/zellijcontroller.go` (modify existing)

Updates:
- Add SessionService dependency: `NewZellijController(z *ZellijService, sessionSvc *session.SessionService)`
- Update `PUT /sessions` handler to call `sessionSvc.UpdateFromZellij(ctx, req.Sessions)`

### Phase 4: Daemon Integration

#### 4.1 Add Error Helper
**File**: `internal/common/error.go` (modify)

Add missing 404 error helper:
```go
func ErrNotFound() render.Renderer {
    return &ErrResponse{
        Err:            errors.New("resource not found"),
        HTTPStatusCode: 404,
        StatusText:     "Resource not found.",
        ErrorText:      "The requested resource does not exist.",
    }
}
```

#### 4.2 Update Daemon
**File**: `internal/api/daemon.go` (modify)

Wire up all modules with lifecycle management using module pattern:

```go
func StartDaemon() {
    ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
    defer stop()

    // 1. Initialize modules in dependency order
    workspaceModule := workspace.NewWorkspaceModule()
    sessionModule := session.NewSessionModule(workspaceModule)
    // Future modules can be added here

    // 2. Collect all modules that implement lifecycle interface
    modules := []common.Module{
        workspaceModule,
        sessionModule,
    }

    // 3. Call OnAppStart for all modules
    for _, mod := range modules {
        if err := mod.OnAppStart(ctx); err != nil {
            log.Fatalf("Failed to start module: %v", err)
        }
    }

    // 4. Ensure OnAppEnd is called on shutdown
    defer func() {
        // Call in reverse order for proper cleanup
        for i := len(modules) - 1; i >= 0; i-- {
            if err := modules[i].OnAppEnd(context.Background()); err != nil {
                log.Printf("Error during module shutdown: %v", err)
            }
        }
    }()

    // 5. Initialize ZellijService (doesn't have module yet)
    zellijSvc := zellij.NewZellijService()

    // 6. Start API server
    go serveAPI(ctx, workspaceModule, sessionModule, zellijSvc)

    <-ctx.Done()
}

func serveAPI(ctx context.Context, workspaceModule *workspace.WorkspaceModule,
              sessionModule *session.SessionModule, zellijSvc *zellij.ZellijService) {
    r := chi.NewRouter()

    r.Use(middleware.RequestID)
    r.Use(middleware.Logger)
    r.Use(middleware.Recoverer)
    r.Use(middleware.URLFormat)
    r.Use(render.SetContentType(render.ContentTypeJSON))

    // Mount module routers
    r.Mount("/sessions", sessionModule.Router)
    r.Mount("/workspaces", workspaceModule.Router)
    r.Mount("/zellij", zellij.NewZellijController(zellijSvc, sessionModule.Service))

    http.ListenAndServe(":3333", r)
}
```

**Key points:**
- Modules instantiated in dependency order (workspace before session)
- `OnAppStart` called after all modules created
- `OnAppEnd` called in reverse order for proper cleanup
- Module routers mounted directly to main router

### Phase 5: Test Coverage

#### 5.1 Workspace Module Tests

**File**: `internal/workspace/workspace_test.go` (new)
- Test Workspace struct JSON marshaling/unmarshaling

**File**: `internal/workspace/workspacestore_test.go` (new)
- Test WorkspaceStore CRUD operations
- Test GetByID, GetByPath
- Test concurrent access with goroutines
- Test thread safety

**File**: `internal/workspace/workspaceservice_test.go` (new)
- Test ListWorkspaces (verify hard-coded workspaces)
- Test GetWorkspace (by ID, not found)
- Test GetWorkspaceByPath (found, not found)
- Test OnAppStart (verify workspaces initialized)
- Test OnAppEnd

**File**: `internal/workspace/workspacecontroller_test.go` (new)
- Test WorkspaceController handlers
- Test ListWorkspaces handler (success, error cases)
- Use httptest.ResponseRecorder for testing

**File**: `internal/workspace/workspacerouter_test.go` (new)
- Test GET /workspaces endpoint (verify hard-coded workspaces returned)
- Test error responses (500)
- Use httptest to test router integration

**File**: `internal/workspace/workspacemodule_test.go` (new)
- Test module initialization
- Test OnAppStart/OnAppEnd lifecycle methods
- Verify all components properly wired together

#### 5.2 Session Module Tests

**File**: `internal/session/session_test.go` (new)
- Test Session struct JSON marshaling/unmarshaling

**File**: `internal/session/sessionstore_test.go` (new)
- Test SessionStore CRUD operations
- Test concurrent access with goroutines
- Test thread safety

**File**: `internal/session/sessionservice_test.go` (new)
- Test ListSessions (empty, single, multiple, MRU order)
- Test GetSession (found, not found)
- Test CreateSession (valid, duplicate name, invalid workspace ID)
- Test ActivateSession (valid, not found, verify LastAccessedAt update)
- Test UpdateFromZellij (new sessions, update active session, mark inactive)
- Test lifecycle methods (OnAppStart, OnAppEnd)

**File**: `internal/session/validation_test.go` (new)
- Test validateSessionName (valid, invalid chars, too long, empty)
- Test generateDefaultSessionName (no collision, with collision, counter, timestamp)

**File**: `internal/session/sessioncontroller_test.go` (new)
- Test SessionController handlers
- Test ListSessions handler (success, error cases)
- Test CreateSession handler (with/without name, validation)
- Test ActivateSession handler (success, not found)
- Use httptest.ResponseRecorder for testing

**File**: `internal/session/sessionrouter_test.go` (new)
- Test GET /sessions (empty, with sessions, MRU order)
- Test POST /sessions (valid, missing workspace ID, invalid workspace ID, duplicate name, empty name generates default)
- Test PUT /sessions/{name}/activate (valid, not found)
- Test error responses (400, 404, 500)
- Use httptest to test router integration

**File**: `internal/session/sessionmodule_test.go` (new)
- Test module initialization with workspace module dependency
- Test OnAppStart/OnAppEnd lifecycle methods
- Verify all components properly wired together

#### 5.3 Integration Tests

**File**: `internal/api/integration_test.go` (new)
- Test full module initialization and lifecycle
- Test full flow: create session, list sessions, activate session
- Test Zellij sync: PUT /zellij/sessions, verify sessions updated
- Test MRU ordering across multiple operations
- Test module dependency injection (session module depends on workspace module)

#### 5.4 Test Utilities

**File**: `internal/common/testutils.go` (new)
- Helper functions for creating test HTTP requests
- Helper functions for parsing responses
- Mock context creation
- Helper for creating test modules

## Implementation Order

**Phase 0: Foundation**
1. **Lifecycle interface** (`common/lifecycle.go`) - Module interface with OnAppStart/OnAppEnd
2. **Error helper** (`common/error.go`) - Add ErrNotFound

**Phase 1: Workspace Module**
3. **Workspace entity** (`workspace/workspace.go`) - Base struct
4. **WorkspaceStore** (`workspace/workspacestore.go`) - Storage layer
5. **WorkspaceStore tests** (`workspace/workspacestore_test.go`)
6. **WorkspaceService** (`workspace/workspaceservice.go`) - Business logic + lifecycle
7. **WorkspaceService tests** (`workspace/workspaceservice_test.go`)
8. **Workspace API types** (`workspace/types.go`) - Request/response structs
9. **WorkspaceController** (`workspace/workspacecontroller.go`) - HTTP handlers
10. **WorkspaceRouter** (`workspace/workspacerouter.go`) - Router setup
11. **WorkspaceRouter tests** (`workspace/workspacerouter_test.go`)
12. **WorkspaceModule** (`workspace/workspacemodule.go`) - Bundle all components

**Phase 2: Session Module**
13. **Session entity** (`session/session.go`) - Base struct
14. **SessionStore** (`session/sessionstore.go`) - Storage layer
15. **SessionStore tests** (`session/sessionstore_test.go`)
16. **Session validation** (`session/validation.go`) - Validation logic
17. **Validation tests** (`session/validation_test.go`)
18. **SessionService** (`session/sessionservice.go`) - Business logic + lifecycle
19. **SessionService tests** (`session/sessionservice_test.go`)
20. **Session API types** (`session/types.go`) - Request/response structs
21. **SessionController** (`session/sessioncontroller.go`) - HTTP handlers
22. **SessionRouter** (`session/sessionrouter.go`) - Router setup
23. **SessionRouter tests** (`session/sessionrouter_test.go`)
24. **SessionModule** (`session/sessionmodule.go`) - Bundle all components

**Phase 3: Integration**
25. **ZellijController update** (`zellij/zellijcontroller.go`) - Integrate SessionService
26. **Daemon integration** (`api/daemon.go`) - Wire modules with lifecycle
27. **Integration tests** (`api/integration_test.go`) - End-to-end tests
28. **Manual API testing** - Verify with curl commands

**Key principles:**
- Build each module completely before moving to next
- Write tests alongside implementation (test-driven development)
- Workspace module has no dependencies (build first)
- Session module depends on workspace module (build second)
- Integration happens last after all modules complete

## Critical Files

### New Files (31 files):

**Common:**
- `internal/common/lifecycle.go` - Generic module interface
- `internal/common/testutils.go` - Test utilities

**Workspace Module (13 files):**
- `internal/workspace/workspace.go` - Workspace entity (replaces existing)
- `internal/workspace/workspacestore.go` - In-memory store
- `internal/workspace/workspaceservice.go` - Service with lifecycle
- `internal/workspace/workspacecontroller.go` - HTTP handlers
- `internal/workspace/workspacerouter.go` - Router setup
- `internal/workspace/types.go` - API types
- `internal/workspace/workspacemodule.go` - Module that bundles all components
- `internal/workspace/workspace_test.go` - Entity tests
- `internal/workspace/workspacestore_test.go` - Store tests
- `internal/workspace/workspaceservice_test.go` - Service tests
- `internal/workspace/workspacecontroller_test.go` - Controller tests
- `internal/workspace/workspacerouter_test.go` - Router tests
- `internal/workspace/workspacemodule_test.go` - Module tests

**Session Module (15 files):**
- `internal/session/session.go` - Session entity
- `internal/session/sessionstore.go` - In-memory store
- `internal/session/sessionservice.go` - Service with lifecycle
- `internal/session/sessioncontroller.go` - HTTP handlers
- `internal/session/sessionrouter.go` - Router setup
- `internal/session/validation.go` - Validation logic
- `internal/session/types.go` - API types
- `internal/session/sessionmodule.go` - Module that bundles all components
- `internal/session/session_test.go` - Entity tests
- `internal/session/sessionstore_test.go` - Store tests
- `internal/session/sessionservice_test.go` - Service tests
- `internal/session/sessioncontroller_test.go` - Controller tests
- `internal/session/sessionrouter_test.go` - Router tests
- `internal/session/validation_test.go` - Validation tests
- `internal/session/sessionmodule_test.go` - Module tests

**Integration:**
- `internal/api/integration_test.go` - End-to-end integration tests

### Modified Files (3 files):
- `internal/common/error.go` - Add ErrNotFound helper
- `internal/zellij/zellijcontroller.go` - Add SessionService integration
- `internal/api/daemon.go` - Wire modules with lifecycle management

### Removed Files (1 file):
- `internal/session/sessioncontroller.go` - Replaced by module router pattern

## Technical Details

### Module Pattern
- Each module exports `New<Name>Module()` function that returns module struct
- Module struct contains: Store, Service, and Router (if applicable)
- Modules implement `common.Module` interface (OnAppStart, OnAppEnd)
- Dependencies passed to module constructor (e.g., SessionModule depends on WorkspaceModule)
- Clean encapsulation: all module components bundled together
- Easy to test: each module can be tested independently

**Controller/Router Separation:**
- **Controller** (`<name>controller.go`): Struct with HTTP handler methods, accepts service dependencies
- **Router** (`<name>router.go`): Instantiates controller and wires handlers to routes
- Benefits: Clear separation of concerns, controller logic testable independently from routing

### Lifecycle Management
- All modules implement `common.Module` interface
- Daemon instantiates modules in dependency order
- After all modules created, daemon calls `OnAppStart(ctx)` on each
- Defers `OnAppEnd(ctx)` for graceful shutdown (reverse order)
- Enables adding new modules easily without daemon changes
- Reverse shutdown order ensures dependencies cleaned up properly

### State Management
- **In-memory only** (no persistence for now)
- Thread safety via `sync.RWMutex` on stores
- Read-heavy workload optimized with RWMutex

### UUID vs Path
- Sessions reference workspaces by UUID (WorkspaceID)
- More durable than path (paths can change)
- Workspace service provides lookup by both ID and path

### MRU Sorting
- `ListSessions()` sorts by `LastAccessedAt` descending
- Update `LastAccessedAt` on activate and Zellij sync

### Hard-coded Workspaces
- 2-3 example workspaces in `WorkspaceService.OnAppStart()`
- Simple IDs like "ws-1", "ws-2"
- Can be replaced with discovery mechanism later

### Session Sync from Zellij
- Plugin sends `PUT /zellij/sessions` with all current sessions
- Create sessions if they don't exist (need workspace ID - TBD)
- Mark session with `is_current_session: true` as active
- Update `LastAccessedAt` for active session
- Mark all others as inactive

**Note:** Zellij sync needs workspace mapping - plugin must send workspace path and daemon must look up workspace by path to get UUID.

### Error Handling
- 201: Session created successfully
- 400: Invalid request (validation errors, duplicate name, invalid workspace ID)
- 404: Session not found, workspace not found
- 500: Internal server error

## Verification

### Unit Tests
Run all unit tests:
```bash
go test ./internal/session/...
go test ./internal/workspace/...
go test ./internal/api/...
```

Verify coverage:
```bash
go test -cover ./internal/session/...
go test -cover ./internal/workspace/...
go test -cover ./internal/api/...
```

### API Testing (with daemon running)
```bash
# Start daemon
task daemon:run

# Test GET /workspaces (should return hard-coded workspaces)
curl http://localhost:3333/workspaces

# Test GET /sessions (should be empty initially)
curl http://localhost:3333/sessions

# Test POST /sessions (create session with workspace ID from above)
curl -X POST http://localhost:3333/sessions \
  -H "Content-Type: application/json" \
  -d '{"name": "test-session", "workspace_id": "ws-1"}'

# Test GET /sessions (should show created session)
curl http://localhost:3333/sessions

# Test PUT /sessions/{name}/activate
curl -X PUT http://localhost:3333/sessions/test-session/activate

# Test GET /sessions (verify LastAccessedAt updated, MRU order)
curl http://localhost:3333/sessions

# Test create with empty name (should generate default)
curl -X POST http://localhost:3333/sessions \
  -H "Content-Type: application/json" \
  -d '{"workspace_id": "ws-1"}'

# Test create with invalid workspace ID (should return 400)
curl -X POST http://localhost:3333/sessions \
  -H "Content-Type: application/json" \
  -d '{"name": "bad-session", "workspace_id": "invalid"}'

# Test activate non-existent session (should return 404)
curl -X PUT http://localhost:3333/sessions/nonexistent/activate
```

### Integration Testing
1. Create multiple sessions with different workspaces
2. Verify MRU ordering (most recent first)
3. Activate different sessions, verify order changes
4. Test Zellij plugin integration: send PUT /zellij/sessions, verify sync
5. Restart daemon, verify sessions are lost (in-memory only)

### Edge Cases
- Create session with duplicate name (should return 400)
- Create session with invalid workspace ID (should return 400)
- Activate non-existent session (should return 404)
- Create session with empty name (should generate default)
- Test concurrent API calls

## Dependencies

### Go Packages
- `github.com/go-chi/chi/v5` - already imported
- `github.com/go-chi/render` - already imported
- Standard library: `sync`, `time`, `context`, `errors`, `fmt`, `regexp`
- Testing: `testing`, `net/http/httptest`

### No New External Dependencies
All implementation uses existing packages and Go standard library.

## Notes

- **Module pattern** - standardized structure for organizing related components
- **Controller/Router separation** - controllers define HTTP handlers, routers wire them to routes
- **No persistence** in this implementation - sessions lost on restart
- **Hard-coded workspaces** - full discovery mechanism deferred
- **Lifecycle interface** enables future modules to integrate cleanly
- **Dependency injection** via module constructors (SessionModule depends on WorkspaceModule)
- **Test-first approach** - comprehensive test coverage required
- **Encapsulation** - each module owns its controller, router, service, and store
- TUI and plugin integration are separate tasks
- Session switching commands (daemon → plugin via pipe) will be implemented when integrating the plugin
- Future modules can be added easily by following the same pattern
