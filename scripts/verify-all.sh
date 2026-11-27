#!/usr/bin/env bash
set -e

echo "===> Running Rust Backend Unit Tests..."
cargo test --manifest-path src-tauri/Cargo.toml

echo "===> Running TypeScript Type Checks..."
npx tsc --noEmit

echo "===> Running Vite Production Build..."
npm run build

echo "===> All checks passed cleanly!"
