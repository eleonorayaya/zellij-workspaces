package zellij

import (
	"context"

	"github.com/eleonorayaya/utena/internal/session"
	"github.com/go-chi/chi/v5"
)

type ZellijModule struct {
	Service    *ZellijService
	Controller *ZellijController
	Router     *ZellijRouter
}

func NewZellijModule(sessionModule *session.SessionModule) *ZellijModule {
	service := NewZellijService(sessionModule.Service)
	controller := NewZellijController(service)
	router := NewZellijRouter(controller)

	return &ZellijModule{
		Service:    service,
		Controller: controller,
		Router:     router,
	}
}

func (m *ZellijModule) OnAppStart(ctx context.Context) error {

	if err := m.Service.OnAppStart(ctx); err != nil {
		return err
	}

	return nil
}

func (m *ZellijModule) OnAppEnd(ctx context.Context) error {

	if err := m.Service.OnAppEnd(ctx); err != nil {
		return err
	}

	return nil
}

func (m *ZellijModule) Routes() chi.Router {
	return m.Router.Routes()
}
