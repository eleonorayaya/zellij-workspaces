package zellij

import (
	"context"

	"github.com/eleonorayaya/utena/internal/eventbus"
)

type ZellijService struct {
	eventBus   eventbus.EventBus
	pipeSender *PipeSender
}

func NewZellijService(bus eventbus.EventBus) *ZellijService {
	return &ZellijService{
		eventBus:   bus,
		pipeSender: NewPipeSender(),
	}
}

func (z *ZellijService) OnAppStart(ctx context.Context) error {
	z.eventBus.Subscribe(eventbus.SessionCreateRequested, z.handleSessionCreateRequested)
	return nil
}

func (z *ZellijService) OnAppEnd(ctx context.Context) error {
	return nil
}

func (z *ZellijService) ProcessSessionUpdate(ctx context.Context, req *UpdateSessionsRequest) error {
	sessions := make([]eventbus.SessionUpdate, len(req.Sessions))
	for i, s := range req.Sessions {
		sessions[i] = eventbus.SessionUpdate{
			Name:             s.Name,
			IsCurrentSession: s.IsCurrentSession,
		}
	}

	event := eventbus.Event{
		Type: eventbus.ZellijSessionsUpdated,
		Data: eventbus.ZellijSessionsUpdatedEvent{
			Sessions: sessions,
		},
	}

	return z.eventBus.Publish(ctx, event)
}

func (z *ZellijService) handleSessionCreateRequested(ctx context.Context, event eventbus.Event) error {
	data, ok := event.Data.(eventbus.SessionCreateRequestedEvent)
	if !ok {
		return nil
	}
	return z.CreateSession(data.SessionName, data.WorkspacePath)
}

func (z *ZellijService) sendCommandToPlugin(command Command) error {
	return z.pipeSender.SendCommand(command)
}

func (z *ZellijService) OpenPicker() error {
	cmd := Command{
		Command: "open_picker",
	}
	return z.sendCommandToPlugin(cmd)
}

func (z *ZellijService) SwitchSession(sessionName string) error {
	cmd := Command{
		Command:     "switch_session",
		SessionName: &sessionName,
	}
	return z.sendCommandToPlugin(cmd)
}

func (z *ZellijService) CreateSession(sessionName, workspacePath string) error {
	cmd := Command{
		Command:       "create_session",
		SessionName:   &sessionName,
		WorkspacePath: &workspacePath,
	}
	return z.sendCommandToPlugin(cmd)
}

func (z *ZellijService) ClosePicker() error {
	cmd := Command{
		Command: "close_picker",
	}
	return z.sendCommandToPlugin(cmd)
}
