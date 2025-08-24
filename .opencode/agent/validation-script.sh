#!/bin/bash

# LLM Success Validation Script
# This script demonstrates how to use the validation agents to prevent false positive success messages

set -euo pipefail

# Configuration
VALIDATION_TIMEOUT=300
STRICT_MODE=true
DEBUG_MODE=false
PROJECT_PATH="."
LLM_OUTPUT_FILE=""
VALIDATION_REPORT="validation_report.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if required tools are installed
check_dependencies() {
    log_info "Checking dependencies..."

    if ! command -v node &> /dev/null; then
        log_error "Node.js is required but not installed"
        exit 1
    fi

    if ! command -v npm &> /dev/null; then
        log_error "npm is required but not installed"
        exit 1
    fi

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo (Rust) is required but not installed"
        exit 1
    fi

    log_success "All dependencies found"
}

# Function to validate LLM output file
validate_llm_output() {
    if [[ -z "$LLM_OUTPUT_FILE" ]]; then
        log_error "LLM output file not specified"
        echo "Usage: $0 --llm-output <file>"
        exit 1
    fi

    if [[ ! -f "$LLM_OUTPUT_FILE" ]]; then
        log_error "LLM output file does not exist: $LLM_OUTPUT_FILE"
        exit 1
    fi

    log_info "Validating LLM output file: $LLM_OUTPUT_FILE"
}

# Function to run output validation
run_output_validation() {
    log_info "Running output validation..."

    local output_validation_result=$(timeout $VALIDATION_TIMEOUT npx ruv-swarm task \
        --description "Validate LLM output completeness and accuracy" \
        --prompt "Please validate the LLM output in file: $LLM_OUTPUT_FILE for completeness, accuracy, and proper error reporting. Check for any false positive success claims." \
        --subagent_type output-validator 2>&1)

    if [[ $? -eq 124 ]]; then
        log_error "Output validation timed out after ${VALIDATION_TIMEOUT}s"
        return 1
    fi

    echo "$output_validation_result" > output_validation.log

    if echo "$output_validation_result" | grep -q "FAILED\|ERROR\|WARNING"; then
        log_warning "Output validation found issues"
        return 1
    else
        log_success "Output validation passed"
        return 0
    fi
}

# Function to run implementation verification
run_implementation_verification() {
    log_info "Running implementation verification..."

    local implementation_result=$(timeout $VALIDATION_TIMEOUT npx ruv-swarm task \
        --description "Verify implementation matches LLM success claims" \
        --prompt "Please verify that the implementation in project: $PROJECT_PATH matches the success claims in LLM output: $LLM_OUTPUT_FILE. Check for compilation errors, test failures, and missing functionality." \
        --subagent_type implementation-verifier 2>&1)

    if [[ $? -eq 124 ]]; then
        log_error "Implementation verification timed out after ${VALIDATION_TIMEOUT}s"
        return 1
    fi

    echo "$implementation_result" > implementation_verification.log

    if echo "$implementation_result" | grep -q "FAILED\|ERROR\|CRITICAL"; then
        log_error "Implementation verification found critical issues"
        return 1
    elif echo "$implementation_result" | grep -q "WARNING"; then
        log_warning "Implementation verification found warnings"
        return 1
    else
        log_success "Implementation verification passed"
        return 0
    fi
}

# Function to run test validation
run_test_validation() {
    log_info "Running test validation..."

    local test_result=$(timeout $VALIDATION_TIMEOUT npx ruv-swarm task \
        --description "Validate test coverage and execution" \
        --prompt "Please validate the test suite in project: $PROJECT_PATH. Check test coverage, execution results, and ensure all tests pass for the functionality claimed in: $LLM_OUTPUT_FILE." \
        --subagent_type test-engineer 2>&1)

    if [[ $? -eq 124 ]]; then
        log_error "Test validation timed out after ${VALIDATION_TIMEOUT}s"
        return 1
    fi

    echo "$test_result" > test_validation.log

    if echo "$test_result" | grep -q "FAILED\|ERROR"; then
        log_error "Test validation found failing tests"
        return 1
    else
        log_success "Test validation passed"
        return 0
    fi
}

# Function to run error handling validation
run_error_validation() {
    log_info "Running error handling validation..."

    local error_result=$(timeout $VALIDATION_TIMEOUT npx ruv-swarm task \
        --description "Validate error handling and reporting" \
        --prompt "Please validate error handling in project: $PROJECT_PATH. Check that all errors are properly caught, reported, and handled according to the claims in: $LLM_OUTPUT_FILE." \
        --subagent_type error-handling 2>&1)

    if [[ $? -eq 124 ]]; then
        log_error "Error validation timed out after ${VALIDATION_TIMEOUT}s"
        return 1
    fi

    echo "$error_result" > error_validation.log

    if echo "$error_result" | grep -q "CRITICAL\|ERROR"; then
        log_error "Error validation found critical issues"
        return 1
    else
        log_success "Error validation passed"
        return 0
    fi
}

# Function to generate comprehensive report
generate_report() {
    log_info "Generating comprehensive validation report..."

    cat > "$VALIDATION_REPORT" << EOF
# LLM Success Validation Report

Generated on: $(date)
Project: $PROJECT_PATH
LLM Output: $LLM_OUTPUT_FILE

## Validation Results

### Summary
EOF

    # Add summary based on validation results
    if [[ -f output_validation.log ]] && [[ -f implementation_verification.log ]]; then
        local overall_status="PASSED"
        local issues_found=0

        if grep -q "FAILED\|ERROR\|WARNING" output_validation.log; then
            overall_status="ISSUES FOUND"
            ((issues_found++))
        fi

        if grep -q "FAILED\|ERROR\|CRITICAL\|WARNING" implementation_verification.log; then
            overall_status="ISSUES FOUND"
            ((issues_found++))
        fi

        if grep -q "FAILED\|ERROR" test_validation.log; then
            overall_status="ISSUES FOUND"
            ((issues_found++))
        fi

        if grep -q "CRITICAL\|ERROR" error_validation.log; then
            overall_status="ISSUES FOUND"
            ((issues_found++))
        fi

        echo "**Overall Status:** $overall_status" >> "$VALIDATION_REPORT"
        echo "**Issues Found:** $issues_found" >> "$VALIDATION_REPORT"
    else
        echo "**Overall Status:** INCOMPLETE" >> "$VALIDATION_REPORT"
        echo "**Issues Found:** Validation incomplete" >> "$VALIDATION_REPORT"
    fi

    # Add detailed results
    cat >> "$VALIDATION_REPORT" << EOF

### Detailed Results

#### Output Validation
$(cat output_validation.log 2>/dev/null || echo "Output validation not completed")

#### Implementation Verification
$(cat implementation_verification.log 2>/dev/null || echo "Implementation verification not completed")

#### Test Validation
$(cat test_validation.log 2>/dev/null || echo "Test validation not completed")

#### Error Handling Validation
$(cat error_validation.log 2>/dev/null || echo "Error validation not completed")

## Recommendations

EOF

    # Generate recommendations based on findings
    if [[ -f output_validation.log ]] && grep -q "FAILED\|ERROR\|WARNING" output_validation.log; then
        echo "- Review and improve LLM output completeness and accuracy" >> "$VALIDATION_REPORT"
    fi

    if [[ -f implementation_verification.log ]] && grep -q "FAILED\|ERROR\|CRITICAL" implementation_verification.log; then
        echo "- Address implementation issues identified in verification" >> "$VALIDATION_REPORT"
    fi

    if [[ -f test_validation.log ]] && grep -q "FAILED\|ERROR" test_validation.log; then
        echo "- Fix failing tests and improve test coverage" >> "$VALIDATION_REPORT"
    fi

    if [[ -f error_validation.log ]] && grep -q "CRITICAL\|ERROR" error_validation.log; then
        echo "- Improve error handling and reporting mechanisms" >> "$VALIDATION_REPORT"
    fi

    log_success "Validation report generated: $VALIDATION_REPORT"
}

# Function to run comprehensive validation
run_comprehensive_validation() {
    local validation_failed=0

    log_info "Starting comprehensive validation..."

    # Run all validations
    if ! run_output_validation; then
        ((validation_failed++))
    fi

    if ! run_implementation_verification; then
        ((validation_failed++))
    fi

    if ! run_test_validation; then
        ((validation_failed++))
    fi

    if ! run_error_validation; then
        ((validation_failed++))
    fi

    # Generate report
    generate_report

    # Final status
    if [[ $validation_failed -gt 0 ]]; then
        log_error "Validation completed with $validation_failed issue(s) found"
        log_info "See $VALIDATION_REPORT for detailed results"
        return 1
    else
        log_success "All validations passed successfully"
        return 0
    fi
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
  --llm-output FILE       Path to LLM output file to validate
  --project PATH          Path to project directory (default: .)
  --report FILE           Output report file (default: validation_report.md)
  --timeout SECONDS       Validation timeout (default: 300)
  --strict                Enable strict validation mode
  --debug                 Enable debug mode
  --help                  Show this help message

Examples:
  $0 --llm-output llm_response.txt
  $0 --llm-output response.md --project /path/to/project --strict
  $0 --llm-output output.txt --timeout 600 --debug

EOF
}

# Main function
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --llm-output)
                LLM_OUTPUT_FILE="$2"
                shift 2
                ;;
            --project)
                PROJECT_PATH="$2"
                shift 2
                ;;
            --report)
                VALIDATION_REPORT="$2"
                shift 2
                ;;
            --timeout)
                VALIDATION_TIMEOUT="$2"
                shift 2
                ;;
            --strict)
                STRICT_MODE=true
                shift
                ;;
            --debug)
                DEBUG_MODE=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Validate arguments
    if [[ -z "$LLM_OUTPUT_FILE" ]]; then
        log_error "LLM output file is required"
        show_usage
        exit 1
    fi

    # Check dependencies
    check_dependencies

    # Validate LLM output file
    validate_llm_output

    # Set debug mode
    if [[ "$DEBUG_MODE" == true ]]; then
        set -x
        log_info "Debug mode enabled"
    fi

    # Run comprehensive validation
    if run_comprehensive_validation; then
        log_success "Validation completed successfully"
        exit 0
    else
        log_error "Validation completed with issues"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"