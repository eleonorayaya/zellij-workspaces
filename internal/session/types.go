package session

import (
	"errors"
	"net/http"

	"github.com/go-chi/render"
)

type SessionResponse struct {
	*Session
}

func NewSessionResponse(session *Session) *SessionResponse {
	return &SessionResponse{Session: session}
}

func (sr *SessionResponse) Render(w http.ResponseWriter, r *http.Request) error {

	return nil
}

type SessionListResponse struct {
	Sessions []Session `json:"sessions"`
}

func NewSessionListResponse(sessions []Session) *SessionListResponse {
	return &SessionListResponse{Sessions: sessions}
}

func (slr *SessionListResponse) Render(w http.ResponseWriter, r *http.Request) error {

	return nil
}

func RenderSessionList(sessions []Session) []render.Renderer {
	list := make([]render.Renderer, len(sessions))
	for i, session := range sessions {
		s := session
		list[i] = NewSessionResponse(&s)
	}
	return list
}

type CreateSessionRequest struct {
	*Session
}

func (c *CreateSessionRequest) Bind(r *http.Request) error {

	if c.Session == nil {
		return errors.New("session cannot be nil")
	}

	return ValidateSession(c.Session)
}

type UpdateSessionRequest struct {
	*Session
}

func (u *UpdateSessionRequest) Bind(r *http.Request) error {

	if u.Session == nil {
		return errors.New("session cannot be nil")
	}

	return ValidateSession(u.Session)
}
