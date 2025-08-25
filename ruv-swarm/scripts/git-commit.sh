#!/bin/bash
# Git Commit Helper Script - Creates commits in new branches with proper conventions

set -e

# Default values
BASE_BRANCH="${BASE_BRANCH:-main}"
PUSH_FLAG=false
PR_FLAG=false
FORCE_FLAG=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

function print_header() {
    echo -e "${BLUE}🔄 Git Commit Helper${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

function validate_branch_name() {
    local branch_name=$1

    # Check if branch name is valid
    if [[ ! $branch_name =~ ^[a-zA-Z0-9][a-zA-Z0-9._-]*$ ]]; then
        echo -e "${RED}❌ Invalid branch name: '$branch_name'${NC}"
        echo -e "${YELLOW}Branch names should:${NC}"
        echo "  - Start with alphanumeric character"
        echo "  - Contain only alphanumeric, dots, underscores, hyphens"
        echo "  - Use kebab-case for consistency"
        return 1
    fi

    # Check for conventional naming
    if [[ ! $branch_name =~ ^(feature|bugfix|hotfix|docs|refactor|test|chore)/ ]]; then
        echo -e "${YELLOW}⚠️  Consider using conventional branch naming:${NC}"
        echo "  - feature/your-feature-name"
        echo "  - bugfix/your-bug-fix"
        echo "  - hotfix/critical-fix"
        echo "  - docs/documentation-update"
        echo "  - refactor/code-improvement"
    fi

    return 0
}

function validate_commit_message() {
    local commit_message=$1

    # Check if message is empty
    if [ -z "$commit_message" ]; then
        echo -e "${RED}❌ Commit message cannot be empty${NC}"
        return 1
    fi

    # Check length
    if [ ${#commit_message} -gt 72 ]; then
        echo -e "${YELLOW}⚠️  Commit message is long (${#commit_message} chars). Consider keeping under 72.${NC}"
    fi

    # Check for conventional commit format
    if [[ ! $commit_message =~ ^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: ]]; then
        echo -e "${YELLOW}💡 Consider using conventional commit format:${NC}"
        echo "  feat: add new feature"
        echo "  fix: resolve bug"
        echo "  docs: update documentation"
        echo "  refactor: improve code structure"
        echo "  test: add tests"
        echo "  chore: maintenance tasks"
    fi

    return 0
}

function check_git_status() {
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}❌ Not in a git repository${NC}"
        exit 1
    fi

    # Check for uncommitted changes
    if [ -z "$(git status --porcelain)" ]; then
        echo -e "${RED}❌ No changes to commit. Stage your changes first:${NC}"
        echo "  git add ."
        echo "  git add <specific-file>"
        exit 1
    fi

    # Check if base branch exists
    if ! git show-ref --verify --quiet "refs/heads/$BASE_BRANCH" && ! git show-ref --verify --quiet "refs/remotes/origin/$BASE_BRANCH"; then
        echo -e "${RED}❌ Base branch '$BASE_BRANCH' does not exist${NC}"
        exit 1
    fi
}

function create_branch() {
    local branch_name=$1

    # Check if branch already exists
    if git show-ref --verify --quiet "refs/heads/$branch_name"; then
        if [ "$FORCE_FLAG" = false ]; then
            echo -e "${RED}❌ Branch '$branch_name' already exists locally${NC}"
            echo -e "${YELLOW}Use --force to overwrite or choose a different name${NC}"
            exit 1
        else
            echo -e "${YELLOW}⚠️  Force creating branch '$branch_name'${NC}"
            git branch -D "$branch_name" 2>/dev/null || true
        fi
    fi

    # Check if remote branch exists
    if git show-ref --verify --quiet "refs/remotes/origin/$branch_name"; then
        if [ "$FORCE_FLAG" = false ]; then
            echo -e "${RED}❌ Branch '$branch_name' already exists on remote${NC}"
            echo -e "${YELLOW}Use --force to overwrite or choose a different name${NC}"
            exit 1
        else
            echo -e "${YELLOW}⚠️  Force creating branch '$branch_name' (remote exists)${NC}"
        fi
    fi

    # Create and switch to new branch
    echo -e "${GREEN}🌱 Creating new branch '$branch_name' from '$BASE_BRANCH'...${NC}"
    git checkout -b "$branch_name" "$BASE_BRANCH"

    echo -e "${GREEN}✅ Branch '$branch_name' created successfully${NC}"
}

function create_commit() {
    local commit_message=$1

    echo -e "${GREEN}📝 Creating commit...${NC}"

    # Create the commit
    if git commit -m "$commit_message"; then
        echo -e "${GREEN}✅ Commit created successfully${NC}"

        # Show commit details
        local commit_hash=$(git rev-parse HEAD)
        echo -e "${BLUE}Commit: ${commit_hash:0:8}${NC}"
        echo -e "${BLUE}Message: $commit_message${NC}"
    else
        echo -e "${RED}❌ Failed to create commit${NC}"
        exit 1
    fi
}

function push_branch() {
    local branch_name=$1

    echo -e "${GREEN}🚀 Pushing branch to remote...${NC}"

    if git push -u origin "$branch_name"; then
        echo -e "${GREEN}✅ Branch pushed successfully${NC}"
        echo -e "${BLUE}Remote branch: origin/$branch_name${NC}"
    else
        echo -e "${RED}❌ Failed to push branch${NC}"
        exit 1
    fi
}

function create_pull_request() {
    local branch_name=$1
    local commit_message=$2

    # Check if GitHub CLI is available
    if ! command -v gh &> /dev/null; then
        echo -e "${YELLOW}⚠️  GitHub CLI not found. Skipping PR creation.${NC}"
        echo -e "${YELLOW}Install with: brew install gh${NC}"
        return
    fi

    # Check if authenticated
    if ! gh auth status &> /dev/null; then
        echo -e "${YELLOW}⚠️  Not authenticated with GitHub CLI. Skipping PR creation.${NC}"
        echo -e "${YELLOW}Run: gh auth login${NC}"
        return
    fi

    echo -e "${GREEN}📋 Creating pull request...${NC}"

    # Extract title from commit message
    local title=$(echo "$commit_message" | head -n 1)

    # Create PR body
    local pr_body="## Changes

$(echo "$commit_message" | tail -n +2)

## Branch
- **From:** $branch_name
- **To:** $BASE_BRANCH

---
*Created automatically by git-commit.sh*"

    # Create the pull request
    if gh pr create --title "$title" --body "$pr_body" --base "$BASE_BRANCH" --head "$branch_name"; then
        echo -e "${GREEN}✅ Pull request created successfully${NC}"
    else
        echo -e "${RED}❌ Failed to create pull request${NC}"
        echo -e "${YELLOW}You can create it manually on GitHub${NC}"
    fi
}

function show_usage() {
    print_header
    echo "Usage: $0 <branch_name> <commit_message> [options]"
    echo
    echo "Arguments:"
    echo "  branch_name     Name of the new branch to create"
    echo "  commit_message  Commit message describing the changes"
    echo
    echo "Options:"
    echo "  --push          Push the branch to remote after commit"
    echo "  --pr            Create a pull request after pushing"
    echo "  --base <branch> Base branch for the new branch (default: main)"
    echo "  --force         Force branch creation even if it exists"
    echo "  --help          Show this help message"
    echo
    echo "Examples:"
    echo "  $0 feature/user-auth \"feat: add user authentication system\""
    echo "  $0 bugfix/login-error \"fix: resolve login validation error\" --push --pr"
    echo "  $0 hotfix/critical-bug \"fix: critical production bug\" --base develop --push"
    echo
    echo "Branch Naming Conventions:"
    echo "  feature/        New features"
    echo "  bugfix/         Bug fixes"
    echo "  hotfix/         Critical fixes"
    echo "  docs/           Documentation"
    echo "  refactor/       Code improvements"
    echo "  test/           Test-related changes"
    echo "  chore/          Maintenance tasks"
}

function parse_arguments() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --push)
                PUSH_FLAG=true
                shift
                ;;
            --pr)
                PR_FLAG=true
                shift
                ;;
            --base)
                BASE_BRANCH="$2"
                shift 2
                ;;
            --force)
                FORCE_FLAG=true
                shift
                ;;
            --help|-h)
                show_usage
                exit 0
                ;;
            -*)
                echo -e "${RED}❌ Unknown option: $1${NC}"
                show_usage
                exit 1
                ;;
            *)
                # Positional arguments
                if [ -z "$BRANCH_NAME" ]; then
                    BRANCH_NAME="$1"
                elif [ -z "$COMMIT_MESSAGE" ]; then
                    COMMIT_MESSAGE="$1"
                else
                    echo -e "${RED}❌ Too many arguments${NC}"
                    show_usage
                    exit 1
                fi
                shift
                ;;
        esac
    done

    # Validate required arguments
    if [ -z "$BRANCH_NAME" ] || [ -z "$COMMIT_MESSAGE" ]; then
        echo -e "${RED}❌ Missing required arguments${NC}"
        show_usage
        exit 1
    fi
}

function main() {
    print_header

    # Parse arguments
    parse_arguments "$@"

    # Validate inputs
    validate_branch_name "$BRANCH_NAME" || exit 1
    validate_commit_message "$COMMIT_MESSAGE" || exit 1

    # Check git status
    check_git_status

    # Create branch
    create_branch "$BRANCH_NAME"

    # Create commit
    create_commit "$COMMIT_MESSAGE"

    # Push if requested
    if [ "$PUSH_FLAG" = true ]; then
        push_branch "$BRANCH_NAME"
    fi

    # Create PR if requested
    if [ "$PR_FLAG" = true ]; then
        if [ "$PUSH_FLAG" = true ]; then
            create_pull_request "$BRANCH_NAME" "$COMMIT_MESSAGE"
        else
            echo -e "${YELLOW}⚠️  Skipping PR creation (requires --push)${NC}"
        fi
    fi

    # Show success message
    echo
    echo -e "${GREEN}🎉 Git commit workflow completed successfully!${NC}"
    echo -e "${BLUE}Branch: $BRANCH_NAME${NC}"
    echo -e "${BLUE}Commit: $(git rev-parse HEAD | cut -c1-8)${NC}"

    if [ "$PUSH_FLAG" = true ]; then
        echo -e "${BLUE}Remote: origin/$BRANCH_NAME${NC}"
    fi

    if [ "$PR_FLAG" = true ] && [ "$PUSH_FLAG" = true ]; then
        echo -e "${BLUE}Next: Check your pull request on GitHub${NC}"
    fi
}

# Run main function with all arguments
main "$@"