package zellij

import (
	"fmt"
	"net/http"

	"github.com/eleonorayaya/utena/internal/common"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/render"
)

type SessionUpdate struct {
	Name             string
	IsCurrentSession bool
}

type UpdateSessionsRequest struct {
	Id       string
	Sessions []SessionUpdate
}

func (s *UpdateSessionsRequest) Bind(r *http.Request) error {
	return nil
}

func NewZellijController(z *ZellijService) chi.Router {
	r := chi.NewRouter()

	r.Put("/sessions", func(w http.ResponseWriter, r *http.Request) {
		req := &UpdateSessionsRequest{}
		if err := render.Bind(r, req); err != nil {
			render.Render(w, r, common.ErrUnknown(err))
			return
		}

		fmt.Printf("Request: %v\n", req)

		if err := z.OnSessionUpdate(r.Context()); err != nil {
			render.Render(w, r, common.ErrUnknown(err))
			return
		}

		render.JSON(w, r, map[string]string{"Ok": "ok"})
	})

	return r
}
