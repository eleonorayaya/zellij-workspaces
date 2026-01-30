package eventbus

import (
	"context"
	"sync"
)

type Event struct {
	Type string
	Data interface{}
}

type Handler func(ctx context.Context, event Event) error

type EventBus interface {
	Subscribe(eventType string, handler Handler)
	Publish(ctx context.Context, event Event) error
}

type InMemoryEventBus struct {
	mu       sync.RWMutex
	handlers map[string][]Handler
}

func NewEventBus() *InMemoryEventBus {
	return &InMemoryEventBus{
		handlers: make(map[string][]Handler),
	}
}

func (bus *InMemoryEventBus) Subscribe(eventType string, handler Handler) {
	bus.mu.Lock()
	defer bus.mu.Unlock()

	bus.handlers[eventType] = append(bus.handlers[eventType], handler)
}

func (bus *InMemoryEventBus) Publish(ctx context.Context, event Event) error {
	bus.mu.RLock()
	handlers := bus.handlers[event.Type]
	bus.mu.RUnlock()

	for _, handler := range handlers {
		if err := handler(ctx, event); err != nil {
			return err
		}
	}

	return nil
}
