package main

import (
	"log"
	"os"

	tea "github.com/charmbracelet/bubbletea"

	"github.com/eleonorayaya/utena/internal/tui"
)

func main() {
	logfilePath := os.Getenv("BUBBLETEA_LOG")
	if logfilePath != "" {
		if _, err := tea.LogToFile(logfilePath, "utena"); err != nil {
			log.Fatal(err)
		}
	}

	p := tea.NewProgram(tui.NewApp())
	if _, err := p.Run(); err != nil {
		log.Fatal(err)
	}
}
