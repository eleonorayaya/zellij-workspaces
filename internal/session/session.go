package session

import "time"

type Session struct {
	ID          string    `json:"id"`
	WorkspaceID string    `json:"workspace_id"`
	IsAttached  bool      `json:"is_attached"`
	IsActive    bool      `json:"is_active"`
	LastUsedAt  time.Time `json:"last_used_at"`
}
