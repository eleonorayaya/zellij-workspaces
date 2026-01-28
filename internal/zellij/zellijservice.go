package zellij

import (
	"context"

	"github.com/eleonorayaya/utena/internal/session"
)

type ZellijService struct {
	sessionService *session.SessionService
}

func NewZellijService(sessionService *session.SessionService) *ZellijService {
	return &ZellijService{
		sessionService: sessionService,
	}
}

func (z *ZellijService) OnAppStart(ctx context.Context) error {

	return nil
}

func (z *ZellijService) OnAppEnd(ctx context.Context) error {

	return nil
}

func (z *ZellijService) ProcessSessionUpdate(ctx context.Context, req *UpdateSessionsRequest) error {
	return nil
}

func (z *ZellijService) UpdateSessionTimestamp(ctx context.Context, sessionID string) error {
	return z.sessionService.UpdateSessionTimestamp(ctx, sessionID)
}
