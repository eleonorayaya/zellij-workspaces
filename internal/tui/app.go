package tui

import (
	"io"
	"log"
	"net/http"
	"time"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	tea "github.com/charmbracelet/bubbletea"
)

type keyMap struct {
	Quit key.Binding
}

func (k keyMap) ShortHelp() []key.Binding {
	return []key.Binding{k.Quit}
}

func (k keyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{k.Quit},
	}
}

type App struct {
	resp string
	keys keyMap
	help help.Model
}

func (a App) Init() tea.Cmd {
	return fetchWorkspaces
}

func (a App) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.QuitMsg:
		return a, tea.Quit

	case tea.KeyMsg:
		switch {
		case key.Matches(msg, a.keys.Quit):
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
	s += "\n\n" + a.help.View(a.keys)
	return s
}

func NewApp() App {
	return App{
		keys: keyMap{
			Quit: key.NewBinding(
				key.WithKeys("q", "ctrl+c"),
				key.WithHelp("q", "quit"),
			),
		},
		help: help.New(),
	}
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
