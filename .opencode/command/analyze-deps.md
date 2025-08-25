---
description: Analyze and optimize project dependencies
agent: orchestrator

---

Analyze dependencies across all projects in the ruv-FANN workspace and identify optimization opportunities.

Dependency Analysis:

1. **Rust Dependencies**:
   - Workspace dependency tree: `!cargo tree --workspace`
   - Unused dependencies: `!cargo +nightly udeps --workspace`
   - Outdated dependencies: `!cargo outdated --workspace`
   - License compatibility: `!cargo license --workspace`

2. **NPM Dependencies**:
   - Package dependency tree: `!npm ls --all`
   - Security vulnerabilities: `!npm audit --audit-level=moderate`
   - Outdated packages: `!npm outdated`
   - Bundle size impact: `!npx webpack-bundle-analyzer package.json`

3. **Cross-Project Dependencies**:
   - Internal crate dependencies: `!cargo tree --workspace | grep -E "(ruv-fann|neuro-divergent|ruv-swarm)"`
   - Circular dependencies: `!cargo +nightly check --workspace --message-format=short | grep "circular"`
   - Version consistency: `!find . -name "Cargo.toml" -exec grep -H "version" {} \;`

4. **Build Dependencies**:
   - Build script dependencies: `!find . -name "build.rs" -exec cat {} \;`
   - Dev dependencies vs runtime: `!cargo tree --workspace --dev`
   - Platform-specific dependencies: `!cargo tree --workspace --target all`

Optimization Recommendations:

- Remove unused dependencies
- Update outdated packages
- Consolidate duplicate dependencies
- Optimize for smaller WASM bundle sizes
- Ensure license compatibility
- Minimize build dependencies

Provide specific commands to fix identified issues.
