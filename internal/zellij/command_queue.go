package zellij

import (
	"sync"
)

type Command struct {
	Command       string  `json:"command"`
	SessionName   *string `json:"session_name,omitempty"`
	WorkspacePath *string `json:"workspace_path,omitempty"`
}

type CommandQueue struct {
	mu       sync.Mutex
	commands []Command
}

func NewCommandQueue() *CommandQueue {
	return &CommandQueue{
		commands: make([]Command, 0),
	}
}

func (q *CommandQueue) Enqueue(cmd Command) {
	q.mu.Lock()
	defer q.mu.Unlock()
	q.commands = append(q.commands, cmd)
}

func (q *CommandQueue) DequeueAll() []Command {
	q.mu.Lock()
	defer q.mu.Unlock()

	commands := q.commands
	q.commands = make([]Command, 0) // Clear queue
	return commands
}
