package eventbus

const (
	ZellijSessionsUpdated  = "zellij.sessions_updated"
	SessionCreateRequested = "session.create_requested"
)

type SessionUpdate struct {
	Name             string
	IsCurrentSession bool
}

type ZellijSessionsUpdatedEvent struct {
	Sessions []SessionUpdate
}

type SessionCreateRequestedEvent struct {
	SessionName   string
	WorkspacePath string
}
