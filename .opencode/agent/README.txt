# OpenCode Agents for ruv-FANN

This directory contains specialized OpenCode agents designed to support the development, maintenance, and operation of the ruv-FANN neural network ecosystem. All agents follow the proper OpenCode agent format with YAML frontmatter.

## Available Agents

### Core Development Agents
- **[git-commit-merge.md](./git-commit-merge.md)** - Git operations, commits, merges, and conflict resolution
- **[github-operations.md](./github-operations.md)** - GitHub repository management, issues, PRs, releases
- **[ci-cd-operations.md](./ci-cd-operations.md)** - Continuous integration and deployment pipelines
- **[rust-core.md](./rust-core.md)** - Rust development and neural network implementation
- **[wasm-engineer.md](./wasm-engineer.md)** - WebAssembly compilation and optimization

### Quality and Reliability Agents
- **[test-engineer.md](./test-engineer.md)** - Comprehensive testing and validation
- **[performance-optimizer.md](./performance-optimizer.md)** - Performance analysis and optimization
- **[memory-leak-prevention.md](./memory-leak-prevention.md)** - Memory management and leak detection
- **[error-handling.md](./error-handling.md)** - Error handling and recovery strategies
- **[security-audit.md](./security-audit.md)** - Security auditing and vulnerability assessment

### Operations and Monitoring Agents
- **[logging.md](./logging.md)** - Logging infrastructure and management
- **[monitoring-observability.md](./monitoring-observability.md)** - System monitoring and observability
- **[command-verifier.md](./command-verifier.md)** - Command verification and documentation

### Architecture and Design Agents
- **[orchestrator.md](./orchestrator.md)** - Multi-project coordination and orchestration
- **[swarm-architect.md](./swarm-architect.md)** - Multi-agent swarm system design
- **[agent-controller.md](./agent-controller.md)** - Multi-agent coordination and management

### Maintenance and Evolution Agents
- **[codebase-cleaner.md](./codebase-cleaner.md)** - Repository cleanup and maintenance
- **[codebase-modernizer.md](./codebase-modernizer.md)** - Codebase analysis and modernization
- **[docs-specialist.md](./docs-specialist.md)** - Technical documentation creation

### Specialized Domain Agents
- **[ml-researcher.md](./ml-researcher.md)** - Machine learning research and implementation
- **[cv-engineer.md](./cv-engineer.md)** - Computer vision and OpenCV integration
- **[opencode-agent-creator.md](./opencode-agent-creator.md)** - OpenCode agent creation

## Agent Format

All agents follow the proper OpenCode agent format:

```yaml
---
description: >-
  Brief description of the agent's purpose and capabilities.
mode: all
---
Agent instructions and content...
```

## How to Use These Agents

### Direct Usage
```bash
# Use specific agents for targeted tasks
npx opencode agent --type github-operations --task "Manage open issues"
npx opencode agent --type memory-leak-prevention --task "Analyze memory usage in training"

# Use agent-controller for complex multi-step operations
npx opencode agent --type agent-controller --task "Prepare v1.0.8 release" \
  --agents "github-operations,ci-cd-operations,security-audit"
```

### Integration with Development Workflow
```bash
# Pre-commit hooks
npx opencode agent --type security-audit --task "Security scan before commit"

# CI/CD integration
npx opencode agent --type ci-cd-operations --task "Optimize build pipeline"

# Code review
npx opencode agent --type test-engineer --task "Review test coverage for new features"
```

## Agent Categories

### 🔧 Development Operations
- `git-commit-merge` - Version control operations
- `github-operations` - Repository management
- `ci-cd-operations` - Build and deployment automation

### 🛡️ Quality Assurance
- `test-engineer` - Testing and validation
- `performance-optimizer` - Performance optimization
- `memory-leak-prevention` - Memory safety
- `error-handling` - Error management
- `security-audit` - Security assessment

### 📊 Observability
- `logging` - Logging infrastructure
- `monitoring-observability` - System monitoring
- `command-verifier` - Command validation

### 🏗️ Architecture
- `orchestrator` - Project coordination
- `swarm-architect` - Swarm system design
- `agent-controller` - Multi-agent management

### 📚 Maintenance
- `codebase-cleaner` - Code cleanup
- `codebase-modernizer` - Code modernization
- `docs-specialist` - Documentation

### 🎯 Domain Specialists
- `rust-core` - Rust development
- `wasm-engineer` - WebAssembly
- `ml-researcher` - Machine learning
- `cv-engineer` - Computer vision

## Best Practices

### Agent Selection
1. **Choose the most specific agent** for your task
2. **Use general agent** for complex, multi-disciplinary tasks
3. **Combine agents** using agent-controller for complex workflows

### Task Definition
- Be specific about the desired outcome
- Include relevant context and constraints
- Specify any integration requirements
- Define success criteria

### Integration
- Integrate agents into your development workflow
- Use agents for code reviews and quality gates
- Automate routine tasks with appropriate agents
- Monitor agent performance and adjust usage

## Contributing

To add new agents or improve existing ones:

1. Follow the established YAML frontmatter format
2. Include comprehensive instructions and examples
3. Define clear integration points with other agents
4. Add appropriate categories and tags
5. Test the agent configuration thoroughly

## Support

For questions about specific agents or general OpenCode usage:
- Check the [OpenCode Documentation](https://opencode.ai/docs/agents/)
- Review existing agent configurations for patterns
- Consult the ruv-FANN development team for project-specific guidance

---

*These agents are designed to enhance the development experience and maintain the high quality standards of the ruv-FANN neural network ecosystem.*