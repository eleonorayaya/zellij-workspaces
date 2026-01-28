package session

import (
	"context"
	"errors"
	"sort"
	"sync"
)

type SessionStore struct {
	mu       sync.RWMutex
	sessions map[string]*Session
}

func NewSessionStore() *SessionStore {
	return &SessionStore{
		sessions: make(map[string]*Session),
	}
}

func (s *SessionStore) GetByID(id string) (*Session, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	session, ok := s.sessions[id]
	if !ok {
		return nil, errors.New("session not found")
	}

	return session, nil
}

func (s *SessionStore) List() []Session {
	s.mu.RLock()
	defer s.mu.RUnlock()

	sessions := make([]Session, 0, len(s.sessions))
	for _, session := range s.sessions {
		sessions = append(sessions, *session)
	}

	sort.Slice(sessions, func(i, j int) bool {
		return sessions[i].LastUsedAt.After(sessions[j].LastUsedAt)
	})

	return sessions
}

func (s *SessionStore) ListByWorkspace(workspaceID string) []Session {
	s.mu.RLock()
	defer s.mu.RUnlock()

	sessions := make([]Session, 0)
	for _, session := range s.sessions {
		if session.WorkspaceID == workspaceID {
			sessions = append(sessions, *session)
		}
	}

	sort.Slice(sessions, func(i, j int) bool {
		return sessions[i].LastUsedAt.After(sessions[j].LastUsedAt)
	})

	return sessions
}

func (s *SessionStore) Add(session *Session) error {
	if session == nil {
		return errors.New("session cannot be nil")
	}

	if session.ID == "" {
		return errors.New("session ID cannot be empty")
	}

	if session.WorkspaceID == "" {
		return errors.New("session WorkspaceID cannot be empty")
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.sessions[session.ID]; exists {
		return errors.New("session with this ID already exists")
	}

	s.sessions[session.ID] = session
	return nil
}

func (s *SessionStore) Update(session *Session) error {
	if session == nil {
		return errors.New("session cannot be nil")
	}

	if session.ID == "" {
		return errors.New("session ID cannot be empty")
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.sessions[session.ID]; !exists {
		return errors.New("session not found")
	}

	s.sessions[session.ID] = session
	return nil
}

func (s *SessionStore) Delete(id string) error {
	if id == "" {
		return errors.New("session ID cannot be empty")
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.sessions[id]; !exists {
		return errors.New("session not found")
	}

	delete(s.sessions, id)
	return nil
}

func (s *SessionStore) OnAppStart(ctx context.Context) error {

	return nil
}

func (s *SessionStore) OnAppEnd(ctx context.Context) error {

	return nil
}
