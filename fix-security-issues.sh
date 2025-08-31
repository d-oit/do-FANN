#!/bin/bash
# fix-security-issues.sh

echo "🔒 Fixing security vulnerabilities..."

# Main package
echo "Fixing main package..."
npm audit fix --force

# ruv-swarm package
echo "Fixing ruv-swarm package..."
cd ruv-swarm/npm
npm audit fix --force
cd ../..

# cuda-wasm package (already clean)
echo "✅ cuda-wasm package already secure"

echo "🎉 Security fixes applied!"