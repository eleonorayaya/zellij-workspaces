package session

import (
	"net/http"

	"github.com/go-chi/chi/v5"
)

func NewSessionController() chi.Router {
	r := chi.NewRouter()
	r.Get("/", func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("sessions"))
	})

	return r
}
