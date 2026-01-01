.PHONY: build run release clean windows linux-arm install-cross targets

# Default build
build:
	cargo build

# Run the application
run:
	cargo run

# Release build for current platform
release:
	cargo build --release

# Install cross
install-cross:
	cargo install cross --git https://github.com/cross-rs/cross

# Cross-compile to Windows (x86_64)
windows:
	cross build --target x86_64-pc-windows-gnu --release

# Cross-compile to Linux ARM64
linux-arm:
	cross build --target aarch64-unknown-linux-gnu --release

# Clean build artifacts
clean:
	cargo clean

# List available targets
targets:
	@echo "Available targets:"
	@echo "  make build      - Debug build for current platform"
	@echo "  make release    - Release build for current platform"
	@echo "  make windows    - Cross-compile to Windows (x86_64)"
	@echo "  make linux-arm  - Cross-compile to Linux ARM64"
	@echo "  make run        - Run the application"
	@echo "  make clean      - Clean build artifacts"
