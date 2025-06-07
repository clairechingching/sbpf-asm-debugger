#!/usr/bin/env bash
set -e

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
CRATE_NAME="helios-vm"
TARGET_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/debug"
WASM_FILE="$TARGET_DIR/${CRATE_NAME//-/_}.wasm"
OUT_DIR="$ROOT_DIR/web/wasm"

echo "🔨 Building $CRATE_NAME to WASM..."
cd "$ROOT_DIR"
cargo build --target wasm32-unknown-unknown --package $CRATE_NAME

echo "📦 Running wasm-bindgen..."
wasm-bindgen --target web --out-dir "$OUT_DIR" "$WASM_FILE"

echo "✅ WASM build and bindgen complete. Output:"
ls -la "$OUT_DIR"
