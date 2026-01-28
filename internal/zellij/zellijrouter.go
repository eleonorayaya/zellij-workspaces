package zellij

import (
	"github.com/go-chi/chi/v5"
)

type ZellijRouter struct {
	controller *ZellijController
}

func NewZellijRouter(controller *ZellijController) *ZellijRouter {
	return &ZellijRouter{
		controller: controller,
	}
}

func (zr *ZellijRouter) Routes() chi.Router {
	r := chi.NewRouter()

	r.Put("/sessions", zr.controller.UpdateSessions)

	return r
}
