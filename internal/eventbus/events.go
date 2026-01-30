package eventbus

const (
	SessionCreateRequested = "session.create_requested"
)

type SessionCreateRequestedEvent struct {
	SessionName   string
	WorkspacePath string
}
