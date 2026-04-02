#!/usr/bin/env bash
# Start the Rust backend server in development mode
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_DIR="$SCRIPT_DIR/server"

echo "Starting Rust backend server (port 8080)..."
cd "$SERVER_DIR"
RUST_LOG=info cargo run --bin server
