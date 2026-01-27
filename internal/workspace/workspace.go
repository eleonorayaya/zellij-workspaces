package workspace

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
)

type Workspace struct {
	Path string
}

type WorkspaceManager struct {
	rootDirs []string
}

type option func(*WorkspaceManager)

func NewWorkspaceManager(opts ...option) *WorkspaceManager {
	w := &WorkspaceManager{}

	for _, opt := range opts {
		opt(w)
	}

	return w
}

func WithRootDir(dir string) option {
	return func(w *WorkspaceManager) {
		w.rootDirs = append(w.rootDirs, dir)
	}
}

func (w *WorkspaceManager) ListWorkspaces(ctx context.Context) ([]Workspace, error) {
	workspaces := make([]Workspace, 0)

	for _, dir := range w.rootDirs {

		entries, err := os.ReadDir(dir)
		if err != nil {
			return workspaces, fmt.Errorf("ListWorkspaces: error reading dir %v: %w", dir, err)
		}

		for _, entry := range entries {
			if entry.IsDir() {
				workspacePath := filepath.Join(dir, entry.Name())
				workspaces = append(workspaces, Workspace{Path: workspacePath})
			}
		}
	}

	return workspaces, nil
}
