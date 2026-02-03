# Utena

A workspace management system for Zellij, consisting of a daemon API server, TUI client, and Zellij plugin.

## Documentation

### Architecture & Patterns

- [Architecture Overview](docs/architecture.md) - System components, module dependencies, and communication patterns
- [Module Structure](docs/module-structure.md) - Standard module layout, layer responsibilities, and lifecycle hooks
- [Event Bus Pattern](docs/event-bus.md) - When and how to use events for module communication

### Development

- [Adding Features](docs/adding-features.md) - Step-by-step guides for adding endpoints, events, and modules

### Project Setup

## Libraries

### Zellij
- [Zellij Plugin API](https://zellij.dev/documentation/plugin-api.html)
- [Zellij Tile Rust Crate](https://docs.rs/zellij-tile/latest/zellij_tile/index.html)

### Bubble Tea
- [Examples](https://github.com/charmbracelet/bubbletea/tree/main/examples)

See [CLAUDE.md](CLAUDE.md) for build commands and development workflow.