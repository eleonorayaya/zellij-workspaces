package workspace

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
)

// setupWorkspaceService creates a workspace service with a fresh store
func setupWorkspaceService(t *testing.T) (*WorkspaceService, *WorkspaceStore) {
	t.Helper()
	store := NewWorkspaceStore()
	service := NewWorkspaceService(store)
	return service, store
}

func TestNewWorkspaceService(t *testing.T) {
	service, _ := setupWorkspaceService(t)
	require.NotNil(t, service)
	require.NotNil(t, service.store)
}

func TestWorkspaceService_OnAppStart(t *testing.T) {
	service, _ := setupWorkspaceService(t)
	ctx := context.Background()
	err := service.OnAppStart(ctx)
	require.NoError(t, err)

	// Service OnAppStart is a no-op
	// Store handles initialization
}

func TestWorkspaceService_OnAppEnd(t *testing.T) {
	service, _ := setupWorkspaceService(t)
	ctx := context.Background()
	err := service.OnAppEnd(ctx)
	require.NoError(t, err)
}

func TestWorkspaceService_ListWorkspaces(t *testing.T) {
	service, store := setupWorkspaceService(t)

	// Add test workspaces
	ws1 := &Workspace{ID: "ws-1", Name: "test1", Path: "/path1"}
	ws2 := &Workspace{ID: "ws-2", Name: "test2", Path: "/path2"}
	store.Add(ws1)
	store.Add(ws2)

	ctx := context.Background()
	workspaces, err := service.ListWorkspaces(ctx)
	require.NoError(t, err)
	require.Len(t, workspaces, 2)

	// Verify both workspaces are in the list
	ids := make(map[string]bool)
	for _, ws := range workspaces {
		ids[ws.ID] = true
	}
	require.True(t, ids["ws-1"])
	require.True(t, ids["ws-2"])
}

func TestWorkspaceService_GetWorkspace(t *testing.T) {
	service, store := setupWorkspaceService(t)

	ws := &Workspace{
		ID:        "ws-1",
		Name:      "test",
		Path:      "/path/to/test",
		IsGitRepo: true,
	}
	store.Add(ws)

	ctx := context.Background()
	retrieved, err := service.GetWorkspace(ctx, "ws-1")
	require.NoError(t, err)
	require.Equal(t, ws.ID, retrieved.ID)
	require.Equal(t, ws.Name, retrieved.Name)
	require.Equal(t, ws.Path, retrieved.Path)
	require.Equal(t, ws.IsGitRepo, retrieved.IsGitRepo)
}

func TestWorkspaceService_GetWorkspace_NotFound(t *testing.T) {
	service, _ := setupWorkspaceService(t)
	ctx := context.Background()
	_, err := service.GetWorkspace(ctx, "nonexistent")
	require.Error(t, err)
	require.Contains(t, err.Error(), "not found")
}

func TestWorkspaceService_GetWorkspaceByPath(t *testing.T) {
	service, store := setupWorkspaceService(t)

	ws := &Workspace{
		ID:   "ws-1",
		Name: "test",
		Path: "/unique/path",
	}
	store.Add(ws)

	ctx := context.Background()
	retrieved, err := service.GetWorkspaceByPath(ctx, "/unique/path")
	require.NoError(t, err)
	require.Equal(t, ws.ID, retrieved.ID)
	require.Equal(t, ws.Path, retrieved.Path)
}

func TestWorkspaceService_GetWorkspaceByPath_NotFound(t *testing.T) {
	service, _ := setupWorkspaceService(t)
	ctx := context.Background()
	_, err := service.GetWorkspaceByPath(ctx, "/nonexistent/path")
	require.Error(t, err)
	require.Contains(t, err.Error(), "not found")
}
