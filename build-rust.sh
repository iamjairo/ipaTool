#!/usr/bin/env bash
# Build the Rust backend server for production
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_DIR="$SCRIPT_DIR/server"

echo "Building Rust backend server..."
cd "$SERVER_DIR"
cargo build --release --bin server
echo "Server binary built at: $SERVER_DIR/target/release/server"
