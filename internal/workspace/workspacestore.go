package workspace

import (
	"context"
	"errors"
	"sync"
)

type WorkspaceStore struct {
	mu         sync.RWMutex
	workspaces map[string]*Workspace
}

func NewWorkspaceStore() *WorkspaceStore {
	return &WorkspaceStore{
		workspaces: make(map[string]*Workspace),
	}
}

func (s *WorkspaceStore) GetByID(id string) (*Workspace, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	ws, ok := s.workspaces[id]
	if !ok {
		return nil, errors.New("workspace not found")
	}

	return ws, nil
}

func (s *WorkspaceStore) GetByPath(path string) (*Workspace, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	for _, ws := range s.workspaces {
		if ws.Path == path {
			return ws, nil
		}
	}

	return nil, errors.New("workspace not found")
}

func (s *WorkspaceStore) List() []Workspace {
	s.mu.RLock()
	defer s.mu.RUnlock()

	workspaces := make([]Workspace, 0, len(s.workspaces))
	for _, ws := range s.workspaces {
		workspaces = append(workspaces, *ws)
	}

	return workspaces
}

func (s *WorkspaceStore) Add(ws *Workspace) error {
	if ws == nil {
		return errors.New("workspace cannot be nil")
	}

	if ws.ID == "" {
		return errors.New("workspace ID cannot be empty")
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.workspaces[ws.ID]; exists {
		return errors.New("workspace with this ID already exists")
	}

	s.workspaces[ws.ID] = ws
	return nil
}

func (s *WorkspaceStore) OnAppStart(ctx context.Context) error {
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
		if err := s.Add(ws); err != nil {
			return err
		}
	}

	return nil
}

func (s *WorkspaceStore) OnAppEnd(ctx context.Context) error {
	return nil
}
