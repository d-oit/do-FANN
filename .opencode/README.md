# OpenCode Commands for ruv-FANN

This directory contains custom OpenCode commands specifically designed for the ruv-FANN multi-project workspace. These commands automate common development tasks and workflows for this complex AI/ML codebase.

## Available Commands

### 🚀 Core Development Commands

#### `/build-workspace`

Build all projects in the workspace with proper dependency order.

- **Use case**: Full workspace compilation
- **Features**: Dependency resolution, error analysis, optimization suggestions

#### `/test-all`

Run comprehensive tests across all projects.

- **Use case**: Complete test suite execution
- **Features**: Rust tests, WASM tests, NPM tests, performance benchmarks, failure analysis

#### `/setup-dev`

Set up complete development environment.

- **Use case**: New developer onboarding
- **Features**: Rust targets, WASM tools, Node.js setup, testing frameworks, IDE configuration

### 🧠 Neural Network Development

#### `/neural-dev`

Neural network development and optimization assistant.

- **Use case**: AI/ML code development and optimization
- **Features**: Architecture analysis, performance optimization, code quality review

#### `/code-review`

AI/ML focused code review for neural network implementations.

- **Use case**: Code quality assessment
- **Features**: Algorithm correctness, performance analysis, security considerations

### 🌐 WebAssembly Commands

#### `/build-wasm`

Build and optimize WebAssembly components.

- **Use case**: WASM compilation and optimization
- **Features**: Bundle size optimization, performance profiling, browser compatibility

### 📊 Performance & Analysis

#### `/benchmark`

Run performance benchmarks and analyze results.

- **Use case**: Performance testing and optimization
- **Features**: Neural network benchmarks, WASM performance, memory analysis

#### `/analyze-deps`

Analyze and optimize project dependencies.

- **Use case**: Dependency management
- **Features**: Unused dependency detection, security scanning, license checking

### 📚 Documentation & Publishing

#### `/generate-docs`

Generate comprehensive documentation for all projects.

- **Use case**: Documentation creation
- **Features**: API docs, architecture docs, usage examples, development guides

#### `/publish-all`

Publish all packages to their respective registries.

- **Use case**: Release management
- **Features**: Crates.io publishing, NPM publishing, Docker images, GitHub releases

## Command Usage

### Running Commands

1. **In OpenCode TUI**: Type `/` followed by the command name

   ```
   /build-workspace
   /test-all
   /neural-dev
   ```

2. **With Arguments**: Some commands accept arguments

   ```
   /component Button
   /analyze-file src/main.rs
   ```

3. **With File References**: Reference specific files
   ```
   /review-component @src/components/Button.tsx
   ```

### Command Features

- **Shell Integration**: Commands can execute shell commands and include output
- **File References**: Include file contents in prompts using `@filename`
- **Agent Assignment**: Each command uses the most appropriate AI agent
- **Model Selection**: Optimized models for different task types

## Command Structure

Each command is a markdown file with frontmatter:

```markdown
---description: Brief description of the commandagent: build---
Command content with instructions and shell commands.
```

## Customization

### Adding New Commands

1. Create a new `.md` file in `.opencode/command/`
2. Add frontmatter with description, agent, and model
3. Write the command instructions
4. Use shell commands with `!command` syntax
5. Reference files with `@filename` syntax

### Modifying Existing Commands

Edit the corresponding `.md` file in `.opencode/command/` to:

- Change the agent or model
- Modify the command instructions
- Add new shell commands or file references
- Update the command description

## Best Practices

1. **Use Specific Agents**: Each command uses the most appropriate agent for the task
2. **Include Context**: Commands gather relevant information before making changes
3. **Handle Errors**: Commands analyze errors and provide solutions
4. **Optimize Performance**: Commands focus on performance and efficiency
5. **Maintain Documentation**: Commands help maintain up-to-date documentation

## Integration with Development Workflow

These commands integrate seamlessly with:

- **Git workflows**: Pre-commit checks, release automation
- **CI/CD pipelines**: Automated testing and deployment
- **Code review**: Automated code analysis and suggestions
- **Documentation**: Auto-generated API and usage documentation

## Troubleshooting

If commands don't work as expected:

1. Check that OpenCode is properly installed
2. Verify the command file syntax and frontmatter
3. Ensure required tools are installed (Rust, Node.js, etc.)
4. Check for any syntax errors in shell commands
5. Review the command output for specific error messages

## Contributing

To contribute new commands:

1. Follow the existing command structure
2. Test the command thoroughly
3. Add appropriate documentation
4. Ensure the command works across all relevant projects
5. Update this README with the new command information