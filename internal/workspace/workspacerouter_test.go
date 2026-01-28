package workspace

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/require"
)

// setupWorkspaceRouter creates and initializes a workspace router with all dependencies
func setupWorkspaceRouter(t *testing.T) (*WorkspaceRouter, *WorkspaceStore) {
	t.Helper()

	store := NewWorkspaceStore()
	ctx := context.Background()
	err := store.OnAppStart(ctx)
	require.NoError(t, err)

	service := NewWorkspaceService(store)
	controller := NewWorkspaceController(service)
	router := NewWorkspaceRouter(controller)

	return router, store
}

func TestWorkspaceRouter_ListWorkspaces(t *testing.T) {
	router, _ := setupWorkspaceRouter(t)

	// Create request
	req := httptest.NewRequest("GET", "/", nil)
	w := httptest.NewRecorder()

	// Execute
	router.Routes().ServeHTTP(w, req)

	// Assert
	require.Equal(t, http.StatusOK, w.Code)

	var response WorkspaceListResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)
	require.Len(t, response.Workspaces, 2)

	// Verify workspace IDs
	ids := make(map[string]bool)
	for _, ws := range response.Workspaces {
		ids[ws.ID] = true
	}
	require.True(t, ids["ws-1"])
	require.True(t, ids["ws-2"])
}

func TestWorkspaceRouter_GetWorkspaceByID(t *testing.T) {
	router, _ := setupWorkspaceRouter(t)

	// Create request
	req := httptest.NewRequest("GET", "/ws-1", nil)
	w := httptest.NewRecorder()

	// Execute
	router.Routes().ServeHTTP(w, req)

	// Assert
	require.Equal(t, http.StatusOK, w.Code)

	var response WorkspaceResponse
	err := json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)
	require.Equal(t, "ws-1", response.ID)
	require.Equal(t, "utena", response.Name)
	require.Equal(t, "/Users/eleonora/dev/utena", response.Path)
	require.True(t, response.IsGitRepo)
}

func TestWorkspaceRouter_GetWorkspaceByID_NotFound(t *testing.T) {
	router, _ := setupWorkspaceRouter(t)

	// Create request
	req := httptest.NewRequest("GET", "/nonexistent", nil)
	w := httptest.NewRecorder()

	// Execute
	router.Routes().ServeHTTP(w, req)

	// Assert
	require.Equal(t, http.StatusNotFound, w.Code)
}
