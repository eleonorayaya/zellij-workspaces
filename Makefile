TARGET = wasm32-wasip1
dev:
	@zellij action new-tab --layout ./zellij.kdl

clean:
	@pkill watchexec

deploy:
	cargo build --target $(TARGET) --release
	@cp $(shell pwd)/target/$(TARGET)/release/zellij-sessionizer.wasm ~/.config/zellij/plugins

