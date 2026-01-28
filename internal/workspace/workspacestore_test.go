package workspace

import (
	"context"
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
)

// setupWorkspaceStore creates a fresh workspace store
func setupWorkspaceStore(t *testing.T) *WorkspaceStore {
	t.Helper()
	return NewWorkspaceStore()
}

func TestNewWorkspaceStore(t *testing.T) {
	store := setupWorkspaceStore(t)
	require.NotNil(t, store)
	require.NotNil(t, store.workspaces)
}

func TestWorkspaceStore_Add(t *testing.T) {
	store := setupWorkspaceStore(t)

	ws := &Workspace{
		ID:        "ws-1",
		Name:      "test",
		Path:      "/path/to/test",
		IsGitRepo: true,
	}

	err := store.Add(ws)
	require.NoError(t, err)

	// Verify workspace was added
	retrieved, err := store.GetByID("ws-1")
	require.NoError(t, err)
	require.Equal(t, ws.ID, retrieved.ID)
}

func TestWorkspaceStore_Add_NilWorkspace(t *testing.T) {
	store := setupWorkspaceStore(t)
	err := store.Add(nil)
	require.Error(t, err)
	require.Contains(t, err.Error(), "cannot be nil")
}

func TestWorkspaceStore_Add_EmptyID(t *testing.T) {
	store := setupWorkspaceStore(t)
	ws := &Workspace{Name: "test", Path: "/path"}
	err := store.Add(ws)
	require.Error(t, err)
	require.Contains(t, err.Error(), "ID cannot be empty")
}

func TestWorkspaceStore_Add_Duplicate(t *testing.T) {
	store := setupWorkspaceStore(t)

	ws1 := &Workspace{ID: "ws-1", Name: "test1", Path: "/path1"}
	ws2 := &Workspace{ID: "ws-1", Name: "test2", Path: "/path2"}

	err := store.Add(ws1)
	require.NoError(t, err)

	err = store.Add(ws2)
	require.Error(t, err)
	require.Contains(t, err.Error(), "already exists")
}

func TestWorkspaceStore_GetByID(t *testing.T) {
	store := setupWorkspaceStore(t)

	ws := &Workspace{
		ID:        "ws-1",
		Name:      "test",
		Path:      "/path/to/test",
		IsGitRepo: false,
	}

	store.Add(ws)

	retrieved, err := store.GetByID("ws-1")
	require.NoError(t, err)
	require.Equal(t, ws.ID, retrieved.ID)
	require.Equal(t, ws.Name, retrieved.Name)
	require.Equal(t, ws.Path, retrieved.Path)
	require.Equal(t, ws.IsGitRepo, retrieved.IsGitRepo)
}

func TestWorkspaceStore_GetByID_NotFound(t *testing.T) {
	store := setupWorkspaceStore(t)

	_, err := store.GetByID("nonexistent")
	require.Error(t, err)
	require.Contains(t, err.Error(), "not found")
}

func TestWorkspaceStore_GetByPath(t *testing.T) {
	store := setupWorkspaceStore(t)

	ws := &Workspace{
		ID:   "ws-1",
		Name: "test",
		Path: "/unique/path",
	}

	store.Add(ws)

	retrieved, err := store.GetByPath("/unique/path")
	require.NoError(t, err)
	require.Equal(t, ws.ID, retrieved.ID)
}

func TestWorkspaceStore_GetByPath_NotFound(t *testing.T) {
	store := setupWorkspaceStore(t)

	_, err := store.GetByPath("/nonexistent/path")
	require.Error(t, err)
	require.Contains(t, err.Error(), "not found")
}

func TestWorkspaceStore_List(t *testing.T) {
	store := setupWorkspaceStore(t)

	ws1 := &Workspace{ID: "ws-1", Name: "test1", Path: "/path1"}
	ws2 := &Workspace{ID: "ws-2", Name: "test2", Path: "/path2"}

	store.Add(ws1)
	store.Add(ws2)

	list := store.List()
	require.Len(t, list, 2)

	// Verify both workspaces are in the list
	ids := make(map[string]bool)
	for _, ws := range list {
		ids[ws.ID] = true
	}

	require.True(t, ids["ws-1"], "ws-1 not found in list")
	require.True(t, ids["ws-2"], "ws-2 not found in list")
}

func TestWorkspaceStore_List_Empty(t *testing.T) {
	store := setupWorkspaceStore(t)
	list := store.List()
	require.Empty(t, list)
}

func TestWorkspaceStore_ConcurrentAccess(t *testing.T) {
	store := setupWorkspaceStore(t)

	var wg sync.WaitGroup
	numGoroutines := 10

	// Concurrent adds
	for i := 0; i < numGoroutines; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			ws := &Workspace{
				ID:   string(rune('a' + id)),
				Name: "concurrent",
				Path: "/path",
			}
			store.Add(ws)
		}(i)
	}

	wg.Wait()

	// Concurrent reads
	for i := 0; i < numGoroutines; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			store.List()
		}()
	}

	wg.Wait()

	// Should have added workspaces without panic
	list := store.List()
	require.Len(t, list, numGoroutines)
}

func TestWorkspaceStore_OnAppStart(t *testing.T) {
	store := setupWorkspaceStore(t)

	ctx := context.Background()
	err := store.OnAppStart(ctx)
	require.NoError(t, err)

	// Verify workspaces were initialized
	workspaces := store.List()
	require.Len(t, workspaces, 2)

	// Check ws-1 (utena)
	ws1, err := store.GetByID("ws-1")
	require.NoError(t, err)
	require.Equal(t, "ws-1", ws1.ID)
	require.Equal(t, "utena", ws1.Name)
	require.Equal(t, "/Users/eleonora/dev/utena", ws1.Path)
	require.True(t, ws1.IsGitRepo)

	// Check ws-2 (example-project)
	ws2, err := store.GetByID("ws-2")
	require.NoError(t, err)
	require.Equal(t, "ws-2", ws2.ID)
	require.Equal(t, "example-project", ws2.Name)
	require.Equal(t, "/Users/eleonora/dev/example", ws2.Path)
	require.False(t, ws2.IsGitRepo)
}

func TestWorkspaceStore_OnAppEnd(t *testing.T) {
	store := setupWorkspaceStore(t)

	ctx := context.Background()
	err := store.OnAppEnd(ctx)
	require.NoError(t, err)
}
