package session

import "errors"

func ValidateSession(session *Session) error {
	if session == nil {
		return errors.New("session cannot be nil")
	}

	if session.ID == "" {
		return errors.New("session ID cannot be empty")
	}

	if session.WorkspaceID == "" {
		return errors.New("session WorkspaceID cannot be empty")
	}

	if session.LastUsedAt.IsZero() {
		return errors.New("session LastUsedAt cannot be zero")
	}

	return nil
}

func ValidateSessionID(id string) error {
	if id == "" {
		return errors.New("session ID cannot be empty")
	}

	return nil
}

func ValidateWorkspaceID(id string) error {
	if id == "" {
		return errors.New("workspace ID cannot be empty")
	}

	return nil
}
