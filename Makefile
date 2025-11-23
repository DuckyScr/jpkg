.PHONY: all build install clean env

# Default target
all: build

# Build the project in debug mode
build:
	cargo build

# Install the binary to the local cargo bin directory
install:
	cargo install --path . --force

# Create a shell script to source for development testing
env:
	@echo "export PATH=\"$$(pwd)/target/debug:\$$PATH\"" > env.sh
	@echo "Created env.sh. Run 'source env.sh' to add the debug binary to your PATH."

# Clean build artifacts
clean:
	cargo clean
