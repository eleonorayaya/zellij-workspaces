package internal

import (
	"context"
	"net/http"

	"github.com/eleonorayaya/utena/internal/session"
	"github.com/eleonorayaya/utena/internal/zellij"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/render"
)

func serveAPI(ctx context.Context, workspaceMgr *WorkspaceManager, zellijSvc *zellij.ZellijService) {
	r := chi.NewRouter()

	r.Use(middleware.RequestID)
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)
	r.Use(middleware.URLFormat)
	r.Use(render.SetContentType(render.ContentTypeJSON))

	r.Mount("/sessions", session.NewSessionController())
	r.Mount("/zellij", zellij.NewZellijController(zellijSvc))

	http.ListenAndServe(":3333", r)
}
