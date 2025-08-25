---
description: Perform git commit with proper analysis and validation
agent: git-commit-merge

---

Perform a comprehensive git commit operation following best practices for the ruv-FANN workspace.

## Commit Process

### Phase 1: Pre-Commit Analysis
1. **Status Check**: Review all staged and unstaged changes
2. **Diff Analysis**: Examine what files have been modified
3. **Recent Commits**: Check commit history for context and style

### Phase 2: Change Analysis
Analyze all changes to determine:
- **Nature of Changes**: New feature, enhancement, bug fix, refactoring, test, docs, etc.
- **Purpose/Motivation**: Why these changes were made
- **Impact**: How changes affect the overall project
- **Security**: Check for sensitive information that shouldn't be committed

### Phase 3: Commit Message Drafting
Create a concise commit message that:
- Focuses on "why" rather than "what"
- Uses clear, concise language
- Accurately reflects the changes and their purpose
- Follows conventional commit format when applicable

### Phase 4: Validation & Commit
- **Pre-commit Hooks**: Ensure all hooks pass
- **Build Verification**: Confirm code still builds
- **Test Validation**: Run relevant tests
- **Security Check**: Final security audit

### Phase 5: Post-Commit Actions
- **Push**: Push to remote repository if needed
- **Status Update**: Confirm commit was successful
- **Documentation**: Update any relevant documentation

## Usage
This command will guide you through the complete commit process, ensuring all changes are properly analyzed, validated, and committed with meaningful messages.

## Requirements
- Git repository must be initialized
- Changes must be staged (or command will help stage them)
- All pre-commit checks must pass

## Error Handling
If pre-commit hooks fail, the command will:
1. Identify the specific issues
2. Suggest fixes where possible
3. Allow retry after fixes are applied
4. Provide clear error messages for manual resolution