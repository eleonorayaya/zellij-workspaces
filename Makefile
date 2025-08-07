TARGET = wasm32-wasip1
BIN = zellij-workspaces.wasm

.PHONY: clean dev reload deploy


clean:
	@pkill watchexec
	@rm -r ~/Library/Caches/org.Zellij-Contributors.Zellij/

reload:
	@zellij action start-or-reload-plugin zellij-workspaces

deploy:
	cargo build --target $(TARGET) --release
	@cp $(shell pwd)/target/$(TARGET)/release/$(BIN) ~/.config/zellij/plugins

