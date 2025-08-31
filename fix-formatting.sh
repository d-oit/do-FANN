#!/bin/bash
# fix-formatting.sh

echo "🎨 Fixing code formatting..."

# ruv-swarm package
echo "Formatting ruv-swarm..."
cd ruv-swarm/npm
npx prettier --write "src/**/*.{js,ts,mjs,cjs}" "test/**/*.{js,ts,mjs,cjs}"
cd ../..

# cuda-wasm package
echo "Formatting cuda-wasm..."
cd cuda-wasm
npx prettier --write "cli/**/*.js" "scripts/**/*.js"
cd ..

echo "✨ Formatting complete!"