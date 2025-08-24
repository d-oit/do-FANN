---
description: Publish all packages to their respective registries
agent: build

---

Publish all packages in the ruv-FANN workspace to their respective registries.

Publishing checklist:

1. **Rust Crates (crates.io)**:
   - Check current versions: `!cargo tree --workspace | grep -E "^[^│├└ ]"`
   - Update version numbers if needed
   - Run tests: `!cargo test --workspace`
   - Publish core crates: `!cargo publish -p ruv-fann`
   - Publish dependent crates: `!cargo publish -p neuro-divergent`

2. **NPM Packages (npmjs.com)**:
   - Check package versions: `!npm view ruv-swarm version`
   - Build WASM packages: `!wasm-pack build cuda-wasm --target nodejs`
   - Update package.json versions
   - Publish npm packages: `!npm publish cuda-wasm/pkg --access public`

3. **Docker Images (if applicable)**:
   - Build images: `!docker build -t ruv-fann:latest .`
   - Tag appropriately: `!docker tag ruv-fann:latest ruvnet/ruv-fann:latest`
   - Push to registry: `!docker push ruvnet/ruv-fann:latest`

4. **GitHub Releases**:
   - Create release tag: `!git tag v1.0.0`
   - Push tags: `!git push origin v1.0.0`
   - Create release notes with changelog

Pre-publishing checks:

- All tests pass
- Documentation is up to date
- Version numbers are consistent
- Dependencies are properly specified
- License information is correct

Handle any publishing errors and provide solutions for common issues like authentication, version conflicts, or dependency problems.
