---
description: Comprehensive git operations for ruv-FANN workspace
agent: orchestrator
---

Comprehensive git operations for the ruv-FANN multi-project workspace. This command provides all essential git functionality with workspace-specific optimizations.

## 📋 Git Status Overview

Current repository status:
`!git status --porcelain`

Branch information:
`!git branch -a`

Recent commits:
`!git log --oneline -5`

## 🚀 Quick Git Operations

### Status & Information
- **Check status**: `!git status`
- **View branches**: `!git branch -v`
- **Recent activity**: `!git log --oneline -10 --graph`
- **Stash list**: `!git stash list`

### Branch Management
- **Create feature branch**: `!git checkout -b feature/$(date +%Y%m%d)-$FEATURE_NAME`
- **Switch branch**: `!git checkout $BRANCH_NAME`
- **Delete branch**: `!git branch -d $BRANCH_NAME`
- **Merge branch**: `!git merge $BRANCH_NAME`

### Commit Operations
- **Stage all changes**: `!git add .`
- **Stage specific files**: `!git add $FILE_PATH`
- **Commit changes**: `!git commit -m "$COMMIT_MESSAGE"`
- **Amend last commit**: `!git commit --amend -m "$NEW_MESSAGE"`

### Remote Operations
- **Push to remote**: `!git push origin $(git branch --show-current)`
- **Pull from remote**: `!git pull origin $(git branch --show-current)`
- **Fetch updates**: `!git fetch --all --prune`
- **View remotes**: `!git remote -v`

## 🏗️ Workspace-Specific Git Workflows

### Multi-Project Coordination
For the ruv-FANN workspace with multiple projects:

1. **Cross-project status**:
   `!find . -name ".git" -type d | while read dir; do echo "=== ${dir%/.git} ==="; cd "$dir/.."; git status --porcelain; cd - > /dev/null; done`

2. **Workspace commit**:
   - Stage all project changes: `!git add .`
   - Commit with workspace context: `!git commit -m "feat: $DESCRIPTION

   Projects updated:
   - ruv-fann: $RUV_FANN_CHANGES
   - neuro-divergent: $NEURO_CHANGES
   - cuda-wasm: $CUDA_CHANGES
   - ruv-swarm: $SWARM_CHANGES
   - opencv-rust: $OPENCV_CHANGES"`

### Release Management
- **Create release branch**: `!git checkout -b release/v$(date +%Y.%m.%d)`
- **Tag version**: `!git tag -a v$VERSION -m "Release v$VERSION"`
- **Push tags**: `!git push origin --tags`

## 🔧 Advanced Git Operations

### Stashing & Cleanup
- **Stash changes**: `!git stash push -m "$STASH_MESSAGE"`
- **Apply stash**: `!git stash pop`
- **Clean workspace**: `!git clean -fd`
- **Reset to HEAD**: `!git reset --hard HEAD`

### History & Investigation
- **Search commits**: `!git log --grep="$SEARCH_TERM"`
- **File history**: `!git log --follow -p $FILE_PATH`
- **Blame file**: `!git blame $FILE_PATH`
- **Show changes**: `!git show $COMMIT_HASH`

### Collaboration
- **Create PR branch**: `!git checkout -b pr/$FEATURE_NAME`
- **Rebase on main**: `!git rebase main`
- **Squash commits**: `!git rebase -i HEAD~n`
- **Cherry-pick**: `!git cherry-pick $COMMIT_HASH`

## 🚨 Emergency Operations

### Recovery Commands
- **Undo last commit**: `!git reset --soft HEAD~1`
- **Hard reset to remote**: `!git reset --hard origin/$(git branch --show-current)`
- **Recover deleted branch**: `!git reflog | head -10`

### Conflict Resolution
- **Show conflicts**: `!git status | grep "both modified"`
- **Resolve conflicts**: Use your preferred merge tool
- **Abort merge**: `!git merge --abort`
- **Continue rebase**: `!git rebase --continue`

## 📊 Git Analytics

### Repository Statistics
- **Contribution stats**: `!git shortlog -sn --all`
- **File change frequency**: `!git log --pretty=format: --name-only | sort | uniq -c | sort -rg | head -20`
- **Author stats**: `!git log --author="$AUTHOR" --pretty=tformat: --numstat | awk '{ add += $1; subs += $2; loc += $1 - $2 } END { printf "added lines: %s, removed lines: %s, total lines: %s\n", add, subs, loc }'`

### Project Health
- **Untracked files**: `!git ls-files --others --exclude-standard`
- **Modified files**: `!git ls-files --modified`
- **Staged files**: `!git diff --cached --name-only`

## ⚙️ Configuration

### Git Configuration for ruv-FANN
```bash
# Set up user info
!git config user.name "$YOUR_NAME"
!git config user.email "$YOUR_EMAIL"

# Set up default branch
!git config init.defaultBranch main

# Enable useful features
!git config core.autocrlf input
!git config pull.rebase true
!git config rebase.autoStash true
```

### Workspace-Specific Settings
- **Ignore patterns**: Check `.gitignore` for workspace-specific ignores
- **Hooks**: Pre-commit hooks for code quality
- **Large file storage**: Git LFS for binary assets

## 🔍 Troubleshooting

### Common Issues
1. **Merge conflicts**: Use `!git status` to identify conflicts, resolve manually
2. **Detached HEAD**: `!git checkout main` to return to main branch
3. **Uncommitted changes**: `!git stash` or `!git commit` to save work
4. **Remote sync issues**: `!git fetch --all --prune` to sync with remote

### Workspace-Specific Tips
- Always check all sub-projects before committing
- Use descriptive commit messages with project prefixes
- Keep feature branches focused and short-lived
- Use `!git submodule status` if working with submodules

For more detailed git operations, use the specific git commands directly or consult the git documentation.