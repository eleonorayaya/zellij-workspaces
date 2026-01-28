package session

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestValidateSession(t *testing.T) {
	tests := []struct {
		name        string
		session     *Session
		expectError bool
		errorMsg    string
	}{
		{
			name: "valid session",
			session: &Session{
				ID:          "session-1",
				WorkspaceID: "ws-1",
				LastUsedAt:  time.Now(),
			},
			expectError: false,
		},
		{
			name:        "nil session",
			session:     nil,
			expectError: true,
			errorMsg:    "cannot be nil",
		},
		{
			name: "empty ID",
			session: &Session{
				WorkspaceID: "ws-1",
				LastUsedAt:  time.Now(),
			},
			expectError: true,
			errorMsg:    "ID cannot be empty",
		},
		{
			name: "empty WorkspaceID",
			session: &Session{
				ID:         "session-1",
				LastUsedAt: time.Now(),
			},
			expectError: true,
			errorMsg:    "WorkspaceID cannot be empty",
		},
		{
			name: "zero LastUsedAt",
			session: &Session{
				ID:          "session-1",
				WorkspaceID: "ws-1",
			},
			expectError: true,
			errorMsg:    "LastUsedAt cannot be zero",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateSession(tt.session)

			if tt.expectError {
				require.Error(t, err)
				require.Contains(t, err.Error(), tt.errorMsg)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateSessionID(t *testing.T) {
	tests := []struct {
		name        string
		id          string
		expectError bool
	}{
		{
			name:        "valid ID",
			id:          "session-1",
			expectError: false,
		},
		{
			name:        "empty ID",
			id:          "",
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateSessionID(tt.id)

			if tt.expectError {
				require.Error(t, err)
				require.Contains(t, err.Error(), "ID cannot be empty")
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestValidateWorkspaceID(t *testing.T) {
	tests := []struct {
		name        string
		id          string
		expectError bool
	}{
		{
			name:        "valid ID",
			id:          "ws-1",
			expectError: false,
		},
		{
			name:        "empty ID",
			id:          "",
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateWorkspaceID(tt.id)

			if tt.expectError {
				require.Error(t, err)
				require.Contains(t, err.Error(), "ID cannot be empty")
			} else {
				require.NoError(t, err)
			}
		})
	}
}
