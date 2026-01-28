package session

import (
	"context"
	"time"

	"github.com/eleonorayaya/utena/internal/workspace"
)

type SessionService struct {
	store          *SessionStore
	workspaceStore *workspace.WorkspaceStore
}

func NewSessionService(store *SessionStore, workspaceStore *workspace.WorkspaceStore) *SessionService {
	return &SessionService{
		store:          store,
		workspaceStore: workspaceStore,
	}
}

func (s *SessionService) OnAppStart(ctx context.Context) error {

	return nil
}

func (s *SessionService) OnAppEnd(ctx context.Context) error {

	return nil
}

func (s *SessionService) ListSessions(ctx context.Context) ([]Session, error) {
	return s.store.List(), nil
}

func (s *SessionService) ListSessionsByWorkspace(ctx context.Context, workspaceID string) ([]Session, error) {

	_, err := s.workspaceStore.GetByID(workspaceID)
	if err != nil {
		return nil, err
	}

	return s.store.ListByWorkspace(workspaceID), nil
}

func (s *SessionService) GetSession(ctx context.Context, id string) (*Session, error) {
	return s.store.GetByID(id)
}

func (s *SessionService) CreateSession(ctx context.Context, session *Session) error {

	_, err := s.workspaceStore.GetByID(session.WorkspaceID)
	if err != nil {
		return err
	}

	if session.LastUsedAt.IsZero() {
		session.LastUsedAt = time.Now()
	}

	return s.store.Add(session)
}

func (s *SessionService) UpdateSession(ctx context.Context, session *Session) error {

	if session.WorkspaceID != "" {
		_, err := s.workspaceStore.GetByID(session.WorkspaceID)
		if err != nil {
			return err
		}
	}

	return s.store.Update(session)
}

func (s *SessionService) DeleteSession(ctx context.Context, id string) error {
	return s.store.Delete(id)
}

func (s *SessionService) UpdateSessionTimestamp(ctx context.Context, id string) error {
	session, err := s.store.GetByID(id)
	if err != nil {
		return err
	}

	session.LastUsedAt = time.Now()
	return s.store.Update(session)
}
