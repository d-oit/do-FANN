#!/bin/bash

#!/bin/bash

# OpenCode Initialization Script for ruv-fann Multi-Project Repository
# This script sets up the OpenCode environment and verifies all configurations

set -e

echo "🚀 Initializing OpenCode for ruv-fann Multi-Project Repository"
echo "================================================================="

# Check if opencode is installed
if ! command -v opencode &> /dev/null; then
    echo "❌ OpenCode is not installed. Please install OpenCode first:"
    echo "   curl -fsSL https://opencode.ai/install | bash"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "opencode.json" ]; then
    echo "❌ Please run this script from the root of the ruv-fann repository"
    exit 1
fi

echo "✅ Found ruv-fann repository"

# Verify configuration files exist
CONFIG_FILES=(
    "opencode.json"
    "OPENCODE_README.md"
)

echo "📋 Verifying configuration files..."
for file in "${CONFIG_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ Missing $file"
        exit 1
    fi
done

# Check if OpenCode is configured
echo "🔧 Checking OpenCode configuration..."
if ! opencode auth status &> /dev/null; then
    echo "⚠️  OpenCode not configured. Please run:"
    echo "   opencode auth login"
    echo ""
    echo "   Then select your preferred AI provider (Anthropic recommended)"
fi

# Verify project structure
echo "🏗️  Verifying project structure..."
PROJECTS=(
    "src"
    "cuda-wasm"
    "ruv-swarm"
    "neuro-divergent"
    "opencv-rust"
    "tests"
    "examples"
)

for project in "${PROJECTS[@]}"; do
    if [ -d "$project" ]; then
        echo "✅ $project/"
    else
        echo "⚠️  $project/ (optional)"
    fi
done

# Check Rust installation
echo "🦀 Checking Rust installation..."
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(cargo --version)
    echo "✅ $RUST_VERSION"
else
    echo "❌ Rust is not installed. Please install Rust 1.81+"
    exit 1
fi

# Check WASM targets
echo "🕸️  Checking WebAssembly targets..."
if rustup target list | grep -q "wasm32-unknown-emscripten (installed)"; then
    echo "✅ wasm32-unknown-emscripten target installed"
else
    echo "⚠️  Installing wasm32-unknown-emscripten target..."
    rustup target add wasm32-unknown-emscripten
fi

if rustup target list | grep -q "wasm32-wasi (installed)"; then
    echo "✅ wasm32-wasi target installed"
else
    echo "⚠️  Installing wasm32-wasi target..."
    rustup target add wasm32-wasi
fi

# Check Node.js installation
echo "📦 Checking Node.js installation..."
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo "✅ Node.js $NODE_VERSION"
else
    echo "❌ Node.js is not installed. Please install Node.js 18.20.8+"
    exit 1
fi

# Check wasm-pack
echo "📦 Checking wasm-pack installation..."
if command -v wasm-pack &> /dev/null; then
    WASM_PACK_VERSION=$(wasm-pack --version)
    echo "✅ $WASM_PACK_VERSION"
else
    echo "⚠️  Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Check npm packages in sub-projects
echo "📦 Checking npm dependencies..."
if [ -d "ruv-swarm/npm" ]; then
    echo "📦 Installing ruv-swarm npm dependencies..."
    cd ruv-swarm/npm
    npm install
    cd ../..
fi

if [ -d "neuro-divergent" ] && [ -f "neuro-divergent/package.json" ]; then
    echo "📦 Installing neuro-divergent npm dependencies..."
    cd neuro-divergent
    npm install
    cd ..
fi

# Verify Cargo workspace
echo "📦 Verifying Cargo workspace..."
if cargo check --workspace --message-format=short; then
    echo "✅ Cargo workspace is valid"
else
    echo "❌ Cargo workspace has issues. Please check the output above."
    exit 1
fi

# Test basic functionality
echo "🧪 Testing basic functionality..."
if cargo build --package ruv-fann --message-format=short; then
    echo "✅ Core library builds successfully"
else
    echo "❌ Core library build failed"
    exit 1
fi

# Test OpenCode functionality
echo "🤖 Testing OpenCode functionality..."
if opencode --version &> /dev/null; then
    echo "✅ OpenCode is working correctly"
else
    echo "❌ OpenCode test failed"
    exit 1
fi

# Generate initial project analysis
echo "📊 Generating initial project analysis..."
echo "   You can now start OpenCode with: opencode"
echo "   Then ask the orchestrator: /ask orchestrator \"Analyze the current project structure\""

echo ""
echo "🎉 OpenCode initialization completed successfully!"
echo "================================================================="
echo ""
echo "📖 Next steps:"
echo "   1. Review the OPENCODE_README.md for detailed usage instructions"
echo "   2. Configure OpenCode: opencode auth login"
echo "   3. Start OpenCode: opencode"
echo "   4. Try asking: /ask orchestrator \"What agents are available?\""
echo ""
echo "🔧 Useful commands:"
echo "   opencode                        # Start OpenCode TUI"
echo "   opencode auth login            # Configure AI provider"
echo "   opencode --help                # Show all available commands"
echo ""
echo "📚 Documentation:"
echo "   OPENCODE_README.md              # Comprehensive usage guide"
echo "   opencode.json                   # Main configuration"
echo ""
echo "🌐 Online Resources:"
echo "   https://opencode.ai/docs       # Official documentation"
echo "   https://opencode.ai/discord    # Community support"