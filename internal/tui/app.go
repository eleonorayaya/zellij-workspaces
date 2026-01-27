package tui

import (
	"io"
	"log"
	"net/http"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

type App struct {
	resp string
}

func (a App) Init() tea.Cmd {
	return fetchWorkspaces
}

func (a App) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.QuitMsg:
		return a, tea.Quit
	case tea.KeyMsg:
		switch msg.String() {

		case "ctrl+c", "q":
			return a, tea.Quit
		}
	case sessionResp:
		a.resp = msg.body
	}

	return a, nil
}

func (a App) View() string {
	s := "hiiiii xD"
	if a.resp != "" {
		s += "\n" + a.resp
	}
	return s
}

func NewApp() App {
	return App{}
}

type sessionResp struct {
	body string
}
type errMsg struct{ error }

func (e errMsg) Error() string { return e.error.Error() }

func fetchWorkspaces() tea.Msg {
	c := &http.Client{
		Timeout: 10 * time.Second,
	}
	res, err := c.Get("http://localhost:3333/sessions")
	if err != nil {
		return errMsg{err}
	}
	defer res.Body.Close()

	bodyBytes, err := io.ReadAll(res.Body)
	if err != nil {
		log.Fatal(err)
	}
	bodyString := string(bodyBytes)
	return sessionResp{bodyString}
}
