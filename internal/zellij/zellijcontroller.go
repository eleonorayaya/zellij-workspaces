package zellij

import (
	"net/http"

	"github.com/eleonorayaya/utena/internal/common"
	"github.com/go-chi/render"
)

type ZellijController struct {
	service *ZellijService
}

func NewZellijController(service *ZellijService) *ZellijController {
	return &ZellijController{
		service: service,
	}
}

func (c *ZellijController) UpdateSessions(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	req := &UpdateSessionsRequest{}
	if err := render.Bind(r, req); err != nil {
		render.Render(w, r, common.ErrInvalidRequest(err))
		return
	}

	if err := c.service.ProcessSessionUpdate(ctx, req); err != nil {
		render.Render(w, r, common.ErrUnknown(err))
		return
	}

	render.JSON(w, r, map[string]string{"status": "ok"})
}
