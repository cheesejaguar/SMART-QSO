#!/bin/bash
#
# SMART-QSO Flight Software Build Verification Script
# Document: SQSO-FSW-SCRIPT-003
# Reference: NPR 7150.2 Section 5.4
#
# Usage: ./verify_build.sh [--flight|--dev|--all]
#
# Exit Codes:
#   0 - All checks passed
#   1 - Build failure
#   2 - Size limit exceeded
#   3 - Static analysis failure
#   4 - Test failure
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$PROJECT_DIR/../.."
BUILD_DIR="$ROOT_DIR/build"
REPORT_DIR="$PROJECT_DIR/reports"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Size limits (bytes)
MAX_TEXT_SIZE=204800      # 200 KB
MAX_DATA_SIZE=20480       # 20 KB
MAX_BSS_SIZE=30720        # 30 KB
MAX_TOTAL_SIZE=262144     # 256 KB

# Exit code
EXIT_CODE=0

echo "=========================================="
echo "SMART-QSO Build Verification"
echo "=========================================="
echo "Project: $PROJECT_DIR"
echo "Build:   $BUILD_DIR"
echo "Date:    $(date)"
echo ""

# Create report directory
mkdir -p "$REPORT_DIR"

# Start verification report
REPORT="$REPORT_DIR/verification_report.txt"
echo "SMART-QSO Build Verification Report" > "$REPORT"
echo "Generated: $(date)" >> "$REPORT"
echo "==========================================" >> "$REPORT"
echo "" >> "$REPORT"

# Function to log result
log_result() {
    local check="$1"
    local status="$2"
    local details="$3"

    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}[PASS]${NC} $check"
        echo "[PASS] $check" >> "$REPORT"
    elif [ "$status" = "WARN" ]; then
        echo -e "${YELLOW}[WARN]${NC} $check: $details"
        echo "[WARN] $check: $details" >> "$REPORT"
    else
        echo -e "${RED}[FAIL]${NC} $check: $details"
        echo "[FAIL] $check: $details" >> "$REPORT"
    fi
}

# Step 1: Clean Build
step_clean_build() {
    echo -e "${BLUE}Step 1: Clean Build${NC}"
    echo "" >> "$REPORT"
    echo "Step 1: Clean Build" >> "$REPORT"

    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"

    if cmake -S "$PROJECT_DIR" -B "$BUILD_DIR" \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_EXPORT_COMPILE_COMMANDS=ON > "$REPORT_DIR/cmake_output.txt" 2>&1; then
        log_result "CMake configuration" "PASS"
    else
        log_result "CMake configuration" "FAIL" "See cmake_output.txt"
        EXIT_CODE=1
        return
    fi

    if cmake --build "$BUILD_DIR" --clean-first > "$REPORT_DIR/build_output.txt" 2>&1; then
        log_result "Build compilation" "PASS"
    else
        log_result "Build compilation" "FAIL" "See build_output.txt"
        EXIT_CODE=1
        return
    fi

    # Check for warnings
    WARN_COUNT=$(grep -c "warning:" "$REPORT_DIR/build_output.txt" 2>/dev/null || echo "0")
    if [ "$WARN_COUNT" -eq 0 ]; then
        log_result "Zero warnings" "PASS"
    else
        log_result "Zero warnings" "WARN" "$WARN_COUNT warnings found"
    fi
}

# Step 2: Size Analysis
step_size_analysis() {
    echo ""
    echo -e "${BLUE}Step 2: Size Analysis${NC}"
    echo "" >> "$REPORT"
    echo "Step 2: Size Analysis" >> "$REPORT"

    # Find the executable
    EXE=$(find "$BUILD_DIR" -name "smart_qso*" -type f -executable 2>/dev/null | head -1)

    if [ -z "$EXE" ]; then
        log_result "Executable found" "FAIL" "No executable found"
        EXIT_CODE=2
        return
    fi

    # Get size info
    if command -v size &> /dev/null; then
        SIZE_OUTPUT=$(size "$EXE" 2>/dev/null || echo "0 0 0")
        echo "Size output: $SIZE_OUTPUT" >> "$REPORT"

        # Parse size (format: text data bss dec hex filename)
        TEXT_SIZE=$(echo "$SIZE_OUTPUT" | tail -1 | awk '{print $1}')
        DATA_SIZE=$(echo "$SIZE_OUTPUT" | tail -1 | awk '{print $2}')
        BSS_SIZE=$(echo "$SIZE_OUTPUT" | tail -1 | awk '{print $3}')
        TOTAL_SIZE=$(echo "$SIZE_OUTPUT" | tail -1 | awk '{print $4}')

        # Validate sizes
        if [ "$TEXT_SIZE" -le "$MAX_TEXT_SIZE" ] 2>/dev/null; then
            log_result "Text size ($TEXT_SIZE B <= $MAX_TEXT_SIZE B)" "PASS"
        else
            log_result "Text size" "FAIL" "$TEXT_SIZE B > $MAX_TEXT_SIZE B"
            EXIT_CODE=2
        fi

        if [ "$DATA_SIZE" -le "$MAX_DATA_SIZE" ] 2>/dev/null; then
            log_result "Data size ($DATA_SIZE B <= $MAX_DATA_SIZE B)" "PASS"
        else
            log_result "Data size" "FAIL" "$DATA_SIZE B > $MAX_DATA_SIZE B"
            EXIT_CODE=2
        fi

        if [ "$BSS_SIZE" -le "$MAX_BSS_SIZE" ] 2>/dev/null; then
            log_result "BSS size ($BSS_SIZE B <= $MAX_BSS_SIZE B)" "PASS"
        else
            log_result "BSS size" "FAIL" "$BSS_SIZE B > $MAX_BSS_SIZE B"
            EXIT_CODE=2
        fi

        # Generate size report
        echo "" >> "$REPORT"
        echo "Size Summary:" >> "$REPORT"
        echo "  Text:  $TEXT_SIZE bytes (limit: $MAX_TEXT_SIZE)" >> "$REPORT"
        echo "  Data:  $DATA_SIZE bytes (limit: $MAX_DATA_SIZE)" >> "$REPORT"
        echo "  BSS:   $BSS_SIZE bytes (limit: $MAX_BSS_SIZE)" >> "$REPORT"
        echo "  Total: $TOTAL_SIZE bytes (limit: $MAX_TOTAL_SIZE)" >> "$REPORT"
    else
        log_result "Size analysis" "WARN" "size command not available"
    fi
}

# Step 3: Static Analysis
step_static_analysis() {
    echo ""
    echo -e "${BLUE}Step 3: Static Analysis${NC}"
    echo "" >> "$REPORT"
    echo "Step 3: Static Analysis" >> "$REPORT"

    # Run cppcheck
    if command -v cppcheck &> /dev/null; then
        cppcheck \
            --enable=warning,style,performance \
            --std=c11 \
            --error-exitcode=1 \
            --suppress=missingIncludeSystem \
            -I"$PROJECT_DIR/include" \
            -I"$PROJECT_DIR/include/hal" \
            "$PROJECT_DIR/src" > "$REPORT_DIR/cppcheck.txt" 2>&1

        if [ $? -eq 0 ]; then
            log_result "cppcheck (no errors)" "PASS"
        else
            log_result "cppcheck" "FAIL" "See cppcheck.txt"
            EXIT_CODE=3
        fi
    else
        log_result "cppcheck" "WARN" "Not installed"
    fi

    # Check for TODO/FIXME in critical code
    TODO_COUNT=$(grep -r "TODO\|FIXME" "$PROJECT_DIR/src" 2>/dev/null | wc -l || echo "0")
    if [ "$TODO_COUNT" -eq 0 ]; then
        log_result "No TODO/FIXME in source" "PASS"
    else
        log_result "TODO/FIXME items" "WARN" "$TODO_COUNT items found"
    fi
}

# Step 4: Run Tests
step_run_tests() {
    echo ""
    echo -e "${BLUE}Step 4: Run Tests${NC}"
    echo "" >> "$REPORT"
    echo "Step 4: Run Tests" >> "$REPORT"

    cd "$BUILD_DIR"

    if [ -f "CTestTestfile.cmake" ]; then
        if ctest --output-on-failure > "$REPORT_DIR/test_output.txt" 2>&1; then
            log_result "Unit tests" "PASS"
        else
            log_result "Unit tests" "FAIL" "See test_output.txt"
            EXIT_CODE=4
        fi
    else
        log_result "Unit tests" "WARN" "No tests configured"
    fi

    cd "$SCRIPT_DIR"
}

# Step 5: Documentation Check
step_doc_check() {
    echo ""
    echo -e "${BLUE}Step 5: Documentation Check${NC}"
    echo "" >> "$REPORT"
    echo "Step 5: Documentation Check" >> "$REPORT"

    # Check for required documentation
    REQUIRED_DOCS=(
        "docs/FDIR_DESIGN.md"
        "docs/TIMING_ANALYSIS.md"
        "docs/MEMORY_POLICY.md"
        "docs/RTOS_ARCHITECTURE.md"
    )

    for doc in "${REQUIRED_DOCS[@]}"; do
        if [ -f "$PROJECT_DIR/$doc" ]; then
            log_result "Doc: $doc" "PASS"
        else
            log_result "Doc: $doc" "WARN" "Missing"
        fi
    done
}

# Step 6: Version Check
step_version_check() {
    echo ""
    echo -e "${BLUE}Step 6: Version Information${NC}"
    echo "" >> "$REPORT"
    echo "Step 6: Version Information" >> "$REPORT"

    # Get git info
    if command -v git &> /dev/null; then
        GIT_HASH=$(git -C "$PROJECT_DIR" rev-parse --short HEAD 2>/dev/null || echo "unknown")
        GIT_BRANCH=$(git -C "$PROJECT_DIR" branch --show-current 2>/dev/null || echo "unknown")
        GIT_DIRTY=$(git -C "$PROJECT_DIR" status --porcelain 2>/dev/null | wc -l)

        echo "  Git hash:   $GIT_HASH" >> "$REPORT"
        echo "  Branch:     $GIT_BRANCH" >> "$REPORT"

        if [ "$GIT_DIRTY" -eq 0 ]; then
            log_result "Clean working tree" "PASS"
        else
            log_result "Clean working tree" "WARN" "$GIT_DIRTY uncommitted changes"
        fi
    fi
}

# Generate summary
generate_summary() {
    echo ""
    echo "=========================================="
    echo "Verification Summary"
    echo "=========================================="
    echo "" >> "$REPORT"
    echo "==========================================" >> "$REPORT"
    echo "Summary" >> "$REPORT"

    PASS_COUNT=$(grep -c "\[PASS\]" "$REPORT" || echo "0")
    WARN_COUNT=$(grep -c "\[WARN\]" "$REPORT" || echo "0")
    FAIL_COUNT=$(grep -c "\[FAIL\]" "$REPORT" || echo "0")

    echo "Passed: $PASS_COUNT"
    echo "Warnings: $WARN_COUNT"
    echo "Failed: $FAIL_COUNT"

    echo "Passed:   $PASS_COUNT" >> "$REPORT"
    echo "Warnings: $WARN_COUNT" >> "$REPORT"
    echo "Failed:   $FAIL_COUNT" >> "$REPORT"

    if [ "$EXIT_CODE" -eq 0 ]; then
        echo ""
        echo -e "${GREEN}BUILD VERIFICATION PASSED${NC}"
        echo "" >> "$REPORT"
        echo "RESULT: PASSED" >> "$REPORT"
    else
        echo ""
        echo -e "${RED}BUILD VERIFICATION FAILED (Exit code: $EXIT_CODE)${NC}"
        echo "" >> "$REPORT"
        echo "RESULT: FAILED (Exit code: $EXIT_CODE)" >> "$REPORT"
    fi

    echo ""
    echo "Report saved: $REPORT"
}

# Main execution
case "${1:-all}" in
    --flight)
        step_clean_build
        step_size_analysis
        step_static_analysis
        step_run_tests
        step_version_check
        generate_summary
        ;;
    --dev)
        step_clean_build
        step_static_analysis
        generate_summary
        ;;
    --all|*)
        step_clean_build
        step_size_analysis
        step_static_analysis
        step_run_tests
        step_doc_check
        step_version_check
        generate_summary
        ;;
esac

exit $EXIT_CODE
