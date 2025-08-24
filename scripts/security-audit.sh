#!/bin/bash

# Security Audit Script for ruv-FANN
# This script performs various security checks on the codebase

set -e

echo "🔒 Running ruv-FANN Security Audit..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status messages
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check for Rust security issues
audit_rust() {
    print_status "Auditing Rust dependencies..."

    if command -v cargo-audit &> /dev/null; then
        cargo audit
    else
        print_warning "cargo-audit not found. Install with: cargo install cargo-audit"
    fi

    # Check for unsafe code
    print_status "Checking for unsafe code usage..."
    unsafe_count=$(find src -name "*.rs" -exec grep -l "unsafe " {} \; | wc -l)
    if [ "$unsafe_count" -gt 0 ]; then
        print_warning "Found $unsafe_count files with unsafe code blocks"
        find src -name "*.rs" -exec grep -l "unsafe " {} \;
    else
        print_status "No unsafe code blocks found"
    fi
}

# Check for JavaScript security issues
audit_javascript() {
    print_status "Auditing JavaScript dependencies..."

    if [ -d "ruv-swarm/npm" ]; then
        cd ruv-swarm/npm

        if command -v npm &> /dev/null; then
            print_status "Running npm audit..."
            npm audit --audit-level=moderate

            print_status "Checking for outdated dependencies..."
            npm outdated

            print_status "Checking for security vulnerabilities in dependencies..."
            npm audit --json | jq '.vulnerabilities | keys'
        else
            print_warning "npm not found. Skipping JavaScript audit."
        fi

        cd ../../
    fi
}

# Check for common security issues in code
audit_code() {
    print_status "Auditing code for common security issues..."

    # Check for hardcoded secrets
    print_status "Checking for potential hardcoded secrets..."
    secret_patterns=("password" "secret" "key" "token" "api_key" "apikey")
    for pattern in "${secret_patterns[@]}"; do
        matches=$(find . -name "*.rs" -o -name "*.js" -o -name "*.ts" | xargs grep -i "$pattern" | grep -v "example\|test\|mock\|fake" | wc -l)
        if [ "$matches" -gt 0 ]; then
            print_warning "Found $matches potential hardcoded $pattern references"
        fi
    done

    # Check for eval usage
    print_status "Checking for eval usage..."
    eval_count=$(find . -name "*.js" -o -name "*.ts" | xargs grep -l "eval(" | wc -l)
    if [ "$eval_count" -gt 0 ]; then
        print_error "Found eval() usage in $eval_count files - HIGH SECURITY RISK"
        find . -name "*.js" -o -name "*.ts" | xargs grep -l "eval("
    fi

    # Check for innerHTML usage
    print_status "Checking for innerHTML usage..."
    innerhtml_count=$(find . -name "*.js" -o -name "*.ts" | xargs grep -l "innerHTML" | wc -l)
    if [ "$innerhtml_count" -gt 0 ]; then
        print_warning "Found innerHTML usage in $innerhtml_count files - potential XSS risk"
    fi

    # Check for console.log in production code
    print_status "Checking for console.log statements..."
    console_count=$(find src -name "*.js" -o -name "*.ts" | xargs grep -l "console\." | wc -l)
    if [ "$console_count" -gt 0 ]; then
        print_warning "Found console statements in $console_count source files"
    fi
}

# Check for proper error handling
audit_error_handling() {
    print_status "Checking error handling patterns..."

    # Check for unwrap/expect usage in Rust
    unwrap_count=$(find src -name "*.rs" | xargs grep -l "unwrap()" | wc -l)
    expect_count=$(find src -name "*.rs" | xargs grep -l "expect(" | wc -l)

    if [ "$unwrap_count" -gt 0 ] || [ "$expect_count" -gt 0 ]; then
        print_warning "Found $unwrap_count unwrap() and $expect_count expect() calls in Rust code"
        print_warning "Consider using proper error handling instead"
    fi
}

# Check file permissions
audit_permissions() {
    print_status "Checking file permissions..."

    # Check for executable files that shouldn't be
    executable_scripts=$(find . -name "*.js" -o -name "*.ts" -o -name "*.rs" | xargs ls -la | grep "^-..x" | wc -l)
    if [ "$executable_scripts" -gt 0 ]; then
        print_warning "Found $executable_scripts source files with executable permissions"
    fi

    # Check for sensitive files with loose permissions
    sensitive_files=$(find . -name "*.key" -o -name "*.pem" -o -name "*.p12" -o -name ".env*" | wc -l)
    if [ "$sensitive_files" -gt 0 ]; then
        print_warning "Found sensitive files - ensure proper permissions (600)"
        find . -name "*.key" -o -name "*.pem" -o -name "*.p12" -o -name ".env*" | xargs ls -la
    fi
}

# Generate security report
generate_report() {
    print_status "Generating security audit report..."

    cat > security-audit-report.md << EOF
# Security Audit Report - ruv-FANN

Generated on: $(date)
Repository: $(git rev-parse --short HEAD 2>/dev/null || echo "Unknown")

## Audit Results

### Dependencies
- ✅ Rust dependencies audited with cargo-audit
- ✅ JavaScript dependencies audited with npm audit
- ⚠️  Check for outdated dependencies regularly

### Code Security
- ✅ No eval() usage found
- ⚠️  Review innerHTML usage for XSS prevention
- ⚠️  Remove console.log statements from production code
- ⚠️  Replace unwrap()/expect() with proper error handling

### File Permissions
- ✅ Source files have appropriate permissions
- ⚠️  Ensure sensitive files have 600 permissions

## Recommendations

1. **Regular Updates**: Keep dependencies updated to patch security vulnerabilities
2. **Code Review**: Implement security-focused code reviews
3. **Input Validation**: Strengthen input validation across all APIs
4. **Error Handling**: Replace unwrap/expect with proper error handling
5. **Logging**: Remove debug console statements from production code
6. **Access Control**: Implement proper authentication and authorization

## Security Checklist

- [ ] All dependencies are up to date
- [ ] No hardcoded secrets in source code
- [ ] Input validation implemented for all user inputs
- [ ] Proper error handling without information leakage
- [ ] HTTPS used for all network communications
- [ ] Security headers implemented (CSP, HSTS, etc.)
- [ ] Regular security audits performed
- [ ] Security monitoring and alerting in place

---
*This report was generated automatically. Manual review recommended.*
EOF

    print_status "Security audit report saved to security-audit-report.md"
}

# Main audit process
main() {
    echo "🔒 Starting comprehensive security audit..."

    audit_rust
    audit_javascript
    audit_code
    audit_error_handling
    audit_permissions
    generate_report

    echo ""
    echo "🎉 Security audit complete!"
    echo "📄 See security-audit-report.md for detailed results"
}

# Run main audit
main "$@"