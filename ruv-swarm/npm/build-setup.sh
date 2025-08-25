#!/bin/bash

# Build Environment Setup Script for ruv-swarm
# This script sets up the complete build environment

set -e

echo "🔧 Setting up ruv-swarm build environment..."

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Update package list
echo "📦 Updating package list..."
apt-get update

# Install essential build tools
echo "🔨 Installing build essentials..."
apt-get install -y \
    build-essential \
    gcc \
    g++ \
    clang \
    pkg-config \
    libssl-dev \
    curl \
    git

# Install Rust if not present
if ! command_exists rustc; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Install wasm-pack
echo "🌐 Installing wasm-pack..."
cargo install wasm-pack

# Verify installations
echo "✅ Verifying installations..."
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "wasm-pack version: $(wasm-pack --version)"
echo "GCC version: $(gcc --version | head -n1)"
echo "Node.js version: $(node --version)"
echo "npm version: $(npm --version)"

echo ""
echo "🎉 Build environment setup complete!"
echo ""
echo "Next steps:"
echo "1. cd ruv-swarm/npm"
echo "2. npm ci"
echo "3. npm run build:all"
echo "4. node scripts/deploy.js"