#!/bin/bash

# Complete Build Environment Setup for ruv-swarm
# Run this script on your local machine with sudo access

set -e

echo "🚀 Starting complete ruv-swarm build environment setup..."
echo "This script will install all necessary dependencies and build the project"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to print step information
print_step() {
    echo ""
    echo "📋 Step $1: $2"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Step 1: Update system and install Node.js
print_step "1" "Installing Node.js 18.20.8+"
if ! command_exists node || [[ "$(node --version)" < "v18.20.8" ]]; then
    echo "Installing Node.js 18.20.8..."
    curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
    sudo apt-get install -y nodejs
else
    echo "✅ Node.js $(node --version) already installed"
fi

# Step 2: Install system build dependencies
print_step "2" "Installing system build dependencies"
echo "Installing build essentials, compilers, and development tools..."
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    gcc \
    g++ \
    clang \
    pkg-config \
    libssl-dev \
    curl \
    git \
    cmake \
    python3-dev \
    libsqlite3-dev

# Step 3: Install Rust
print_step "3" "Installing Rust toolchain"
if ! command_exists rustc; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    export PATH="$HOME/.cargo/bin:$PATH"
else
    echo "✅ Rust $(rustc --version) already installed"
fi

# Step 4: Update Rust and install wasm-pack
print_step "4" "Setting up Rust development tools"
source ~/.cargo/env
export PATH="$HOME/.cargo/bin:$PATH"

echo "Updating Rust..."
rustup update

echo "Installing wasm-pack..."
cargo install wasm-pack

echo "Installing cargo-watch for development..."
cargo install cargo-watch

# Step 5: Verify installations
print_step "5" "Verifying all installations"
echo "Checking installed versions..."

echo "Node.js: $(node --version)"
echo "npm: $(npm --version)"
echo "Rust: $(rustc --version)"
echo "Cargo: $(cargo --version)"
echo "wasm-pack: $(wasm-pack --version)"
echo "GCC: $(gcc --version | head -n1)"
echo "Clang: $(clang --version | head -n1)"

# Step 6: Clone and setup project
print_step "6" "Setting up ruv-swarm project"
if [[ ! -d "ruv-swarm" ]]; then
    echo "Cloning ruv-swarm repository..."
    git clone https://github.com/ruvnet/ruv-FANN.git
    cd ruv-FANN/ruv-swarm/npm
else
    echo "Project directory exists, navigating to npm directory..."
    cd ruv-swarm/npm
fi

# Step 7: Install Node.js dependencies
print_step "7" "Installing Node.js dependencies"
echo "Installing project dependencies..."
npm ci

# Step 8: Build all components
print_step "8" "Building all components"
echo "Building WASM modules..."
npm run build:wasm

echo "Building SIMD WASM modules..."
npm run build:wasm-simd

echo "Building optimized WASM modules..."
npm run build:wasm-opt

echo "Building JavaScript bundle..."
npm run build

# Step 9: Run tests
print_step "9" "Running test suite"
echo "Running comprehensive tests..."
npm run test:all

# Step 10: Run health checks
print_step "10" "Running health checks"
echo "Running health verification..."
node scripts/health-check.js

# Step 11: Create deployment package
print_step "11" "Creating deployment package"
echo "Creating production deployment package..."
node scripts/deploy.js

echo ""
echo "🎉 BUILD AND DEPLOYMENT COMPLETE!"
echo ""
echo "Your ruv-swarm application is now:"
echo "✅ Built and tested"
echo "✅ Production-ready"
echo "✅ Deployment package created"
echo ""
echo "Next steps:"
echo "1. Check the generated deployment artifacts in ./deploy/"
echo "2. Run 'docker-compose up -d' for containerized deployment"
echo "3. Or run 'npm start' for direct deployment"
echo "4. Access the application at http://localhost:3000"
echo ""
echo "For monitoring and health checks:"
echo "• Health endpoint: http://localhost:3000/health"
echo "• Run 'node scripts/health-check.js --server' for detailed monitoring"
echo ""
echo "Happy deploying! 🚀"