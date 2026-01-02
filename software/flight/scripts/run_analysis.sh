#!/bin/bash
#
# SMART-QSO Flight Software Static Analysis Runner
# Document: SQSO-FSW-SCRIPT-001
# Reference: NPR 7150.2, MISRA C:2012
#
# Usage: ./run_analysis.sh [--full|--quick|--misra]
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SRC_DIR="$PROJECT_DIR/src"
INCLUDE_DIR="$PROJECT_DIR/include"
BUILD_DIR="$PROJECT_DIR/../../build"
REPORT_DIR="$PROJECT_DIR/reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create report directory
mkdir -p "$REPORT_DIR"

echo "=========================================="
echo "SMART-QSO Static Analysis Runner"
echo "=========================================="
echo "Project: $PROJECT_DIR"
echo "Source:  $SRC_DIR"
echo "Reports: $REPORT_DIR"
echo ""

# Function to run cppcheck
run_cppcheck() {
    echo -e "${YELLOW}Running cppcheck...${NC}"

    if ! command -v cppcheck &> /dev/null; then
        echo -e "${RED}cppcheck not found, skipping${NC}"
        return 1
    fi

    cppcheck \
        --enable=all \
        --std=c11 \
        --platform=unix32 \
        --suppress=missingIncludeSystem \
        --suppress=unmatchedSuppression \
        --inline-suppr \
        -I"$INCLUDE_DIR" \
        -I"$INCLUDE_DIR/hal" \
        --xml \
        --xml-version=2 \
        "$SRC_DIR" 2> "$REPORT_DIR/cppcheck.xml"

    # Also generate text report
    cppcheck \
        --enable=all \
        --std=c11 \
        --platform=unix32 \
        --suppress=missingIncludeSystem \
        --suppress=unmatchedSuppression \
        --inline-suppr \
        -I"$INCLUDE_DIR" \
        -I"$INCLUDE_DIR/hal" \
        "$SRC_DIR" 2>&1 | tee "$REPORT_DIR/cppcheck.txt"

    echo -e "${GREEN}cppcheck complete: $REPORT_DIR/cppcheck.xml${NC}"
}

# Function to run clang-tidy
run_clang_tidy() {
    echo -e "${YELLOW}Running clang-tidy...${NC}"

    if ! command -v clang-tidy &> /dev/null; then
        echo -e "${RED}clang-tidy not found, skipping${NC}"
        return 1
    fi

    # Build compile_commands.json if not present
    if [ ! -f "$BUILD_DIR/compile_commands.json" ]; then
        echo "Generating compile_commands.json..."
        cmake -S "$PROJECT_DIR" -B "$BUILD_DIR" -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
    fi

    # Run clang-tidy on all source files
    find "$SRC_DIR" -name "*.c" -print0 | while IFS= read -r -d '' file; do
        echo "  Analyzing: $(basename "$file")"
        clang-tidy \
            -p "$BUILD_DIR" \
            --config-file="$PROJECT_DIR/.clang-tidy" \
            "$file" 2>&1
    done | tee "$REPORT_DIR/clang-tidy.txt"

    echo -e "${GREEN}clang-tidy complete: $REPORT_DIR/clang-tidy.txt${NC}"
}

# Function to check cyclomatic complexity
run_complexity() {
    echo -e "${YELLOW}Running complexity analysis...${NC}"

    if ! command -v lizard &> /dev/null; then
        echo -e "${RED}lizard not found, skipping (pip install lizard)${NC}"
        return 1
    fi

    lizard \
        --CCN 15 \
        --length 200 \
        --arguments 6 \
        --warnings_only \
        "$SRC_DIR" | tee "$REPORT_DIR/complexity.txt"

    # Generate full report
    lizard \
        --CCN 15 \
        --csv \
        "$SRC_DIR" > "$REPORT_DIR/complexity.csv"

    echo -e "${GREEN}Complexity analysis complete: $REPORT_DIR/complexity.csv${NC}"
}

# Function to check stack usage
run_stack_analysis() {
    echo -e "${YELLOW}Running stack analysis...${NC}"

    # Compile with stack usage reporting
    if [ -f "$BUILD_DIR/Makefile" ]; then
        # GCC stack usage
        find "$SRC_DIR" -name "*.c" -print0 | while IFS= read -r -d '' file; do
            base=$(basename "$file" .c)
            gcc -fstack-usage -c "$file" \
                -I"$INCLUDE_DIR" \
                -I"$INCLUDE_DIR/hal" \
                -o "/tmp/$base.o" 2>/dev/null || true
            if [ -f "/tmp/$base.su" ]; then
                cat "/tmp/$base.su" >> "$REPORT_DIR/stack_usage.txt"
            fi
        done
        echo -e "${GREEN}Stack analysis complete: $REPORT_DIR/stack_usage.txt${NC}"
    else
        echo -e "${YELLOW}Build directory not found, skipping stack analysis${NC}"
    fi
}

# Function for MISRA-focused analysis
run_misra() {
    echo -e "${YELLOW}Running MISRA-focused analysis...${NC}"

    if ! command -v cppcheck &> /dev/null; then
        echo -e "${RED}cppcheck not found, skipping${NC}"
        return 1
    fi

    # cppcheck with MISRA addon
    cppcheck \
        --addon=misra \
        --std=c11 \
        --platform=unix32 \
        -I"$INCLUDE_DIR" \
        -I"$INCLUDE_DIR/hal" \
        "$SRC_DIR" 2>&1 | tee "$REPORT_DIR/misra.txt"

    echo -e "${GREEN}MISRA analysis complete: $REPORT_DIR/misra.txt${NC}"
}

# Function to generate summary
generate_summary() {
    echo ""
    echo "=========================================="
    echo "Analysis Summary"
    echo "=========================================="

    REPORT="$REPORT_DIR/summary.txt"
    echo "SMART-QSO Static Analysis Summary" > "$REPORT"
    echo "Generated: $(date)" >> "$REPORT"
    echo "" >> "$REPORT"

    # Count issues from each tool
    if [ -f "$REPORT_DIR/cppcheck.txt" ]; then
        CPPCHECK_ERRORS=$(grep -c "error:" "$REPORT_DIR/cppcheck.txt" 2>/dev/null || echo "0")
        CPPCHECK_WARNINGS=$(grep -c "warning:" "$REPORT_DIR/cppcheck.txt" 2>/dev/null || echo "0")
        echo "cppcheck: $CPPCHECK_ERRORS errors, $CPPCHECK_WARNINGS warnings" | tee -a "$REPORT"
    fi

    if [ -f "$REPORT_DIR/clang-tidy.txt" ]; then
        CLANG_WARNINGS=$(grep -c "warning:" "$REPORT_DIR/clang-tidy.txt" 2>/dev/null || echo "0")
        echo "clang-tidy: $CLANG_WARNINGS warnings" | tee -a "$REPORT"
    fi

    if [ -f "$REPORT_DIR/complexity.txt" ]; then
        COMPLEX_FUNCS=$(wc -l < "$REPORT_DIR/complexity.txt" 2>/dev/null || echo "0")
        echo "Complexity violations: $COMPLEX_FUNCS functions" | tee -a "$REPORT"
    fi

    echo ""
    echo "Reports saved to: $REPORT_DIR/"
}

# Main execution
case "${1:-full}" in
    --quick)
        run_cppcheck
        generate_summary
        ;;
    --misra)
        run_misra
        generate_summary
        ;;
    --full|*)
        run_cppcheck
        run_clang_tidy
        run_complexity
        run_stack_analysis
        generate_summary
        ;;
esac

echo ""
echo -e "${GREEN}Analysis complete!${NC}"
