#!/bin/bash

# Build Environment Setup Script for ruv-FANN
# This script helps set up the development environment with necessary tools

set -e

echo "🔧 Setting up ruv-FANN build environment..."

# Check if we're on a supported platform
OS=$(uname -s)
ARCH=$(uname -m)

echo "Detected: $OS $ARCH"

# Install C compiler and build tools
install_build_tools() {
    echo "📦 Installing build tools..."

    if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        sudo apt-get update
        sudo apt-get install -y build-essential clang llvm-dev libssl-dev pkg-config
    elif command -v yum &> /dev/null; then
        # RHEL/CentOS/Fedora
        sudo yum groupinstall -y "Development Tools"
        sudo yum install -y clang llvm-devel openssl-devel pkgconfig
    elif command -v pacman &> /dev/null; then
        # Arch Linux
        sudo pacman -S --noconfirm base-devel clang llvm openssl pkgconf
    elif command -v brew &> /dev/null; then
        # macOS with Homebrew
        brew install llvm clang openssl pkg-config
    else
        echo "❌ Unsupported package manager. Please install build tools manually:"
        echo "   - C compiler (gcc or clang)"
        echo "   - LLVM development tools"
        echo "   - OpenSSL development libraries"
        echo "   - pkg-config"
        exit 1
    fi
}

# Install Rust if not present
install_rust() {
    if ! command -v cargo &> /dev/null; then
        echo "🦀 Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    else
        echo "✅ Rust is already installed"
    fi

    # Install WASM target
    rustup target add wasm32-unknown-unknown
    rustup target add wasm32-wasi

    # Install useful components
    rustup component add rustfmt
    rustup component add clippy
}

# Install Node.js dependencies
install_node_deps() {
    if command -v npm &> /dev/null; then
        echo "📦 Installing Node.js dependencies..."
        cd ruv-swarm/npm
        npm install
        cd ../../
    else
        echo "⚠️  npm not found. Please install Node.js to work with JavaScript components"
    fi
}

# Set up development tools
setup_dev_tools() {
    echo "🔧 Setting up development tools..."

    # Install cargo tools
    cargo install cargo-audit
    cargo install cargo-tarpaulin
    cargo install wasm-pack
    cargo install cargo-flamegraph

    echo "✅ Development tools installed"
}

# Main setup process
main() {
    echo "🚀 Starting ruv-FANN development environment setup..."

    install_build_tools
    install_rust
    install_node_deps
    setup_dev_tools

    echo ""
    echo "🎉 Setup complete! You can now build ruv-FANN."
    echo ""
    echo "Quick start:"
    echo "  cd ruv-swarm && cargo check    # Check Rust code"
    echo "  cd ruv-swarm/npm && npm test   # Run JavaScript tests"
    echo "  cargo build --release          # Build release version"
    echo ""
    echo "For more information, see CODEBASE_ANALYSIS_IMPROVEMENTS.md"
}

# Run main setup
main "$@"