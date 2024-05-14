# Binary name
BINARY_NAME=lr_webchat_rs

.PHONY: lint

lint:
	@echo "Linting..."
	cargo clippy

build:
	@echo "Building $(BINARY_NAME)..."
	cargo build --release --bin $(BINARY_NAME)
