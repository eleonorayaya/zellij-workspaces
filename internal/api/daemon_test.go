package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/eleonorayaya/utena/internal/session"
	"github.com/eleonorayaya/utena/internal/workspace"
	"github.com/eleonorayaya/utena/internal/zellij"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/render"
	"github.com/stretchr/testify/require"
)

// setupTestRouter creates a test router with all modules initialized
func setupTestRouter(t *testing.T) chi.Router {
	t.Helper()

	ctx := context.Background()

	// Initialize modules
	workspaceModule := workspace.NewWorkspaceModule()
	sessionModule := session.NewSessionModule(workspaceModule)
	zellijModule := zellij.NewZellijModule(sessionModule)

	// Call OnAppStart for all modules
	err := workspaceModule.OnAppStart(ctx)
	require.NoError(t, err)

	err = sessionModule.OnAppStart(ctx)
	require.NoError(t, err)

	err = zellijModule.OnAppStart(ctx)
	require.NoError(t, err)

	// Setup router
	r := chi.NewRouter()
	r.Use(middleware.RequestID)
	r.Use(middleware.Recoverer)
	r.Use(middleware.URLFormat)
	r.Use(render.SetContentType(render.ContentTypeJSON))

	// Mount module routers
	r.Mount("/workspaces", workspaceModule.Routes())
	r.Mount("/sessions", sessionModule.Routes())
	r.Mount("/zellij", zellijModule.Routes())

	return r
}

func TestDaemon_ListWorkspaces(t *testing.T) {
	router := setupTestRouter(t)

	req := httptest.NewRequest("GET", "/workspaces", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	require.Equal(t, http.StatusOK, w.Code)

	var response workspace.WorkspaceListResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)
	require.Len(t, response.Workspaces, 2)

	// Verify hard-coded workspaces
	ids := make(map[string]bool)
	for _, ws := range response.Workspaces {
		ids[ws.ID] = true
	}
	require.True(t, ids["ws-1"])
	require.True(t, ids["ws-2"])
}

func TestDaemon_GetWorkspaceByID(t *testing.T) {
	router := setupTestRouter(t)

	req := httptest.NewRequest("GET", "/workspaces/ws-1", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	require.Equal(t, http.StatusOK, w.Code)

	var response workspace.WorkspaceResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)
	require.Equal(t, "ws-1", response.ID)
	require.Equal(t, "utena", response.Name)
}

func TestDaemon_CreateAndGetSession(t *testing.T) {
	router := setupTestRouter(t)

	// Create session
	sess := &session.Session{
		ID:          "test-session-1",
		WorkspaceID: "ws-1",
		IsAttached:  true,
		IsActive:    true,
		LastUsedAt:  time.Now(),
	}
	body, err := json.Marshal(sess)
	require.NoError(t, err)

	req := httptest.NewRequest("POST", "/sessions", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	require.Equal(t, http.StatusCreated, w.Code)

	// Get session
	req = httptest.NewRequest("GET", "/sessions/test-session-1", nil)
	w = httptest.NewRecorder()

	router.ServeHTTP(w, req)

	require.Equal(t, http.StatusOK, w.Code)

	var response session.SessionResponse
	err = json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)
	require.Equal(t, "test-session-1", response.ID)
	require.Equal(t, "ws-1", response.WorkspaceID)
	require.True(t, response.IsAttached)
}

func TestDaemon_ListSessions(t *testing.T) {
	router := setupTestRouter(t)

	// Create multiple sessions
	sessions := []*session.Session{
		{
			ID:          "session-1",
			WorkspaceID: "ws-1",
			LastUsedAt:  time.Now().Add(-1 * time.Hour),
		},
		{
			ID:          "session-2",
			WorkspaceID: "ws-2",
			LastUsedAt:  time.Now(),
		},
	}

	for _, sess := range sessions {
		body, err := json.Marshal(sess)
		require.NoError(t, err)

		req := httptest.NewRequest("POST", "/sessions", bytes.NewReader(body))
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()

		router.ServeHTTP(w, req)
		require.Equal(t, http.StatusCreated, w.Code)
	}

	// List sessions
	req := httptest.NewRequest("GET", "/sessions", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	require.Equal(t, http.StatusOK, w.Code)

	var response session.SessionListResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)
	require.Len(t, response.Sessions, 2)

	// Verify MRU sorting (most recent first)
	require.Equal(t, "session-2", response.Sessions[0].ID)
	require.Equal(t, "session-1", response.Sessions[1].ID)
}

func TestDaemon_ZellijSessionUpdate(t *testing.T) {
	router := setupTestRouter(t)

	// Send session update from Zellij plugin
	updateReq := &zellij.UpdateSessionsRequest{
		Sessions: []zellij.SessionUpdate{
			{
				Name:             "main-session",
				IsCurrentSession: true,
			},
			{
				Name:             "background-session",
				IsCurrentSession: false,
			},
		},
	}

	body, err := json.Marshal(updateReq)
	require.NoError(t, err)

	req := httptest.NewRequest("PUT", "/zellij/sessions", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	require.Equal(t, http.StatusOK, w.Code)

	var response map[string]string
	err = json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)
	require.Equal(t, "ok", response["status"])
}
