---
description: Show all available OpenCode commands for this workspace
---
Display all available OpenCode commands for the ruv-FANN workspace. Only output the commands. Do not build or implement anything.

Available Commands:

## 🚀 Core Development

- **/build-workspace**: Build all projects with dependency order
- **/test-all**: Run comprehensive test suite across all projects
- **/setup-dev**: Set up complete development environment

## 🧠 Neural Network Development

- **/neural-dev**: Neural network development and optimization
- **/code-review**: AI/ML focused code review

## 🌐 WebAssembly

- **/build-wasm**: Build and optimize WASM components

## 📊 Performance & Analysis

- **/benchmark**: Run performance benchmarks
- **/analyze-deps**: Analyze and optimize dependencies

## 📚 Documentation & Publishing

- **/generate-docs**: Generate comprehensive documentation
- **/publish-all**: Publish to all registries

## 🔧 Utility Commands

- **/git-commit**: Perform git commit with proper analysis and validation
- **/git-operations**: Comprehensive git operations for workspace
- **/help-commands**: Show this command reference

Command Files:
`!find .opencode/command -name "*.md" | sort`

To use any command, type `/` followed by the command name in the OpenCode interface.

For more detailed information about each command, read the corresponding `.md` file in `.opencode/command/`.
