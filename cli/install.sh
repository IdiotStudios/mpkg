#!/usr/bin/env bash
set -e

# === Config ===
BIN_NAME="mpkg"
INSTALL_DIR="/usr/local/bin"

echo "Installing $BIN_NAME to $INSTALL_DIR ..."
sudo install -Dm755 "./$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
echo "✅ Installed to $INSTALL_DIR/$BIN_NAME"
echo "Run: $BIN_NAME --help"