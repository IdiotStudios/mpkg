#!/usr/bin/env bash
set -e

BIN_NAME="mpkg"
INSTALL_DIR="/c/Program Files/$BIN_NAME"



echo "Installing $BIN_NAME.exe to $INSTALL_DIR ..."
mkdir -p "$INSTALL_DIR"
install -Dm755 "./$BIN_NAME.exe" "$INSTALL_DIR/$BIN_NAME.exe"
echo "✅ Installed to $INSTALL_DIR/$BIN_NAME.exe"
echo "⚠️  Make sure '$INSTALL_DIR' is in your PATH."
echo "Run: $BIN_NAME.exe --help"