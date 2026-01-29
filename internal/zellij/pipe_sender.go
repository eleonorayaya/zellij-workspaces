package zellij

import (
	"encoding/json"
	"fmt"
	"os/exec"
)

type PipeSender struct {
	pipeName string
}

func NewPipeSender() *PipeSender {
	return &PipeSender{
		pipeName: "utena-commands",
	}
}

func (p *PipeSender) SendCommand(cmd Command) error {
	// Serialize command to JSON
	payload, err := json.Marshal(cmd)
	if err != nil {
		return fmt.Errorf("failed to marshal command: %w", err)
	}

	// Execute: zellij pipe --name utena-commands --payload '<json>'
	shellCmd := exec.Command(
		"zellij",
		"pipe",
		"--name", p.pipeName,
		"--payload", string(payload),
	)

	output, err := shellCmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("zellij pipe failed: %w, output: %s", err, output)
	}

	return nil
}
