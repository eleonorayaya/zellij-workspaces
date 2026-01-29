package session

import (
	"context"
	"time"

	"github.com/eleonorayaya/utena/internal/eventbus"
	"github.com/eleonorayaya/utena/internal/workspace"
)

type SessionService struct {
	store          *SessionStore
	workspaceStore *workspace.WorkspaceStore
	eventBus       eventbus.EventBus
}

func NewSessionService(store *SessionStore, workspaceStore *workspace.WorkspaceStore, bus eventbus.EventBus) *SessionService {
	return &SessionService{
		store:          store,
		workspaceStore: workspaceStore,
		eventBus:       bus,
	}
}

func (s *SessionService) OnAppStart(ctx context.Context) error {
	s.eventBus.Subscribe(eventbus.ZellijSessionsUpdated, s.handleZellijSessionsUpdated)
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

	if err := s.store.Add(session); err != nil {
		return err
	}

	event := eventbus.Event{
		Type: eventbus.SessionCreateRequested,
		Data: eventbus.SessionCreateRequestedEvent{
			SessionName:   session.ID,
			WorkspacePath: "",
		},
	}
	s.eventBus.Publish(ctx, event)

	return nil
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

func (s *SessionService) handleZellijSessionsUpdated(ctx context.Context, event eventbus.Event) error {
	data, ok := event.Data.(eventbus.ZellijSessionsUpdatedEvent)
	if !ok {
		return nil
	}

	activeSessions := make(map[string]eventbus.SessionUpdate)
	for _, sessionUpdate := range data.Sessions {
		activeSessions[sessionUpdate.Name] = sessionUpdate
	}

	allSessions, err := s.ListSessions(ctx)
	if err != nil {
		return err
	}

	for _, existingSession := range allSessions {
		sess := existingSession

		if update, exists := activeSessions[sess.ID]; exists {
			sess.IsAttached = update.IsCurrentSession
			sess.IsActive = true
			sess.IsDead = false
			sess.LastUsedAt = time.Now()
			delete(activeSessions, sess.ID)
		} else {
			sess.IsDead = true
		}

		if err := s.UpdateSession(ctx, &sess); err != nil {
			return err
		}
	}

	for sessionID, sessionUpdate := range activeSessions {
		newSession := &Session{
			ID:          sessionID,
			WorkspaceID: "ws-1",
			IsAttached:  sessionUpdate.IsCurrentSession,
			IsActive:    true,
			IsDead:      false,
			LastUsedAt:  time.Now(),
		}

		if err := s.CreateSession(ctx, newSession); err != nil {
			return err
		}
	}

	return nil
}
