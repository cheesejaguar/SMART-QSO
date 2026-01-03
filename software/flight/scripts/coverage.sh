#!/bin/bash
#
# SMART-QSO Flight Software Code Coverage Report Generator
# Document: SQSO-FSW-SCRIPT-002
# Reference: NPR 7150.2 Section 5.3
#
# Usage: ./coverage.sh [--html|--summary|--lcov]
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$PROJECT_DIR/../.."
BUILD_DIR="$ROOT_DIR/build-coverage"
REPORT_DIR="$PROJECT_DIR/coverage"
SRC_DIR="$PROJECT_DIR/src"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Coverage thresholds (per NPR 7150.2)
LINE_THRESHOLD=80
BRANCH_THRESHOLD=70
FUNCTION_THRESHOLD=90

echo "=========================================="
echo "SMART-QSO Code Coverage Generator"
echo "=========================================="
echo "Project:    $PROJECT_DIR"
echo "Build:      $BUILD_DIR"
echo "Reports:    $REPORT_DIR"
echo ""

# Check for required tools
check_tools() {
    local missing=0

    if ! command -v gcov &> /dev/null; then
        echo -e "${RED}Error: gcov not found${NC}"
        missing=1
    fi

    if ! command -v lcov &> /dev/null; then
        echo -e "${RED}Error: lcov not found (brew install lcov)${NC}"
        missing=1
    fi

    if ! command -v genhtml &> /dev/null; then
        echo -e "${RED}Error: genhtml not found (part of lcov)${NC}"
        missing=1
    fi

    if [ $missing -eq 1 ]; then
        exit 1
    fi
}

# Build with coverage
build_with_coverage() {
    echo -e "${YELLOW}Building with coverage instrumentation...${NC}"

    mkdir -p "$BUILD_DIR"

    cmake -S "$PROJECT_DIR" -B "$BUILD_DIR" \
        -DCMAKE_BUILD_TYPE=Debug \
        -DENABLE_COVERAGE=ON \
        -DCMAKE_C_FLAGS="--coverage -fprofile-arcs -ftest-coverage" \
        -DCMAKE_EXE_LINKER_FLAGS="--coverage"

    cmake --build "$BUILD_DIR" --clean-first

    echo -e "${GREEN}Build complete${NC}"
}

# Run tests
run_tests() {
    echo -e "${YELLOW}Running tests...${NC}"

    cd "$BUILD_DIR"

    # Run ctest if available
    if [ -f "CTestTestfile.cmake" ]; then
        ctest --output-on-failure || true
    fi

    # Run test executable directly if present
    if [ -f "test_smart_qso" ]; then
        ./test_smart_qso || true
    fi

    cd "$SCRIPT_DIR"
    echo -e "${GREEN}Tests complete${NC}"
}

# Collect coverage data
collect_coverage() {
    echo -e "${YELLOW}Collecting coverage data...${NC}"

    mkdir -p "$REPORT_DIR"

    # Initialize lcov
    lcov --capture \
        --directory "$BUILD_DIR" \
        --output-file "$REPORT_DIR/coverage.info" \
        --ignore-errors gcov,source

    # Remove system headers and test files from coverage
    lcov --remove "$REPORT_DIR/coverage.info" \
        '/usr/*' \
        '*/test/*' \
        '*/tests/*' \
        '*_test.c' \
        --output-file "$REPORT_DIR/coverage_filtered.info" \
        --ignore-errors unused

    echo -e "${GREEN}Coverage data collected${NC}"
}

# Generate HTML report
generate_html() {
    echo -e "${YELLOW}Generating HTML report...${NC}"

    genhtml "$REPORT_DIR/coverage_filtered.info" \
        --output-directory "$REPORT_DIR/html" \
        --title "SMART-QSO Flight Software Coverage" \
        --legend \
        --show-details \
        --highlight \
        --demangle-cpp

    echo -e "${GREEN}HTML report: $REPORT_DIR/html/index.html${NC}"
}

# Generate summary
generate_summary() {
    echo ""
    echo "=========================================="
    echo "Coverage Summary"
    echo "=========================================="

    # Extract metrics from lcov
    if [ -f "$REPORT_DIR/coverage_filtered.info" ]; then
        lcov --summary "$REPORT_DIR/coverage_filtered.info" 2>&1 | tee "$REPORT_DIR/summary.txt"
    fi

    # Parse metrics
    if [ -f "$REPORT_DIR/summary.txt" ]; then
        LINE_COV=$(grep "lines" "$REPORT_DIR/summary.txt" | grep -oP '\d+\.\d+' | head -1 || echo "0")
        FUNC_COV=$(grep "functions" "$REPORT_DIR/summary.txt" | grep -oP '\d+\.\d+' | head -1 || echo "0")
        BRANCH_COV=$(grep "branches" "$REPORT_DIR/summary.txt" | grep -oP '\d+\.\d+' | head -1 || echo "0")

        echo ""
        echo "Threshold Compliance:"
        echo "---------------------"

        # Check line coverage
        if (( $(echo "$LINE_COV >= $LINE_THRESHOLD" | bc -l) )); then
            echo -e "Line coverage:     ${GREEN}${LINE_COV}% >= ${LINE_THRESHOLD}% PASS${NC}"
        else
            echo -e "Line coverage:     ${RED}${LINE_COV}% < ${LINE_THRESHOLD}% FAIL${NC}"
        fi

        # Check function coverage
        if (( $(echo "$FUNC_COV >= $FUNCTION_THRESHOLD" | bc -l) )); then
            echo -e "Function coverage: ${GREEN}${FUNC_COV}% >= ${FUNCTION_THRESHOLD}% PASS${NC}"
        else
            echo -e "Function coverage: ${RED}${FUNC_COV}% < ${FUNCTION_THRESHOLD}% FAIL${NC}"
        fi

        # Check branch coverage
        if (( $(echo "$BRANCH_COV >= $BRANCH_THRESHOLD" | bc -l) )); then
            echo -e "Branch coverage:   ${GREEN}${BRANCH_COV}% >= ${BRANCH_THRESHOLD}% PASS${NC}"
        else
            echo -e "Branch coverage:   ${RED}${BRANCH_COV}% < ${BRANCH_THRESHOLD}% FAIL${NC}"
        fi
    fi
}

# Generate per-file report
generate_file_report() {
    echo -e "${YELLOW}Generating per-file coverage report...${NC}"

    REPORT="$REPORT_DIR/file_coverage.txt"
    echo "SMART-QSO Per-File Coverage Report" > "$REPORT"
    echo "Generated: $(date)" >> "$REPORT"
    echo "" >> "$REPORT"
    echo "File                          Lines    Funcs   Branches" >> "$REPORT"
    echo "----                          -----    -----   --------" >> "$REPORT"

    # Parse lcov info file
    if [ -f "$REPORT_DIR/coverage_filtered.info" ]; then
        lcov --list "$REPORT_DIR/coverage_filtered.info" 2>/dev/null | \
            grep -E "\.c$" | \
            while read line; do
                echo "$line" >> "$REPORT"
            done
    fi

    echo -e "${GREEN}File report: $REPORT_DIR/file_coverage.txt${NC}"
}

# Clean coverage data
clean_coverage() {
    echo -e "${YELLOW}Cleaning coverage data...${NC}"

    rm -rf "$BUILD_DIR"
    rm -rf "$REPORT_DIR"

    # Clean gcov files from source
    find "$SRC_DIR" -name "*.gcno" -delete 2>/dev/null || true
    find "$SRC_DIR" -name "*.gcda" -delete 2>/dev/null || true

    echo -e "${GREEN}Coverage data cleaned${NC}"
}

# Export for CI
export_ci() {
    echo -e "${YELLOW}Exporting CI artifacts...${NC}"

    # Create JSON summary for CI
    if [ -f "$REPORT_DIR/summary.txt" ]; then
        LINE_COV=$(grep "lines" "$REPORT_DIR/summary.txt" | grep -oP '\d+\.\d+' | head -1 || echo "0")
        FUNC_COV=$(grep "functions" "$REPORT_DIR/summary.txt" | grep -oP '\d+\.\d+' | head -1 || echo "0")
        BRANCH_COV=$(grep "branches" "$REPORT_DIR/summary.txt" | grep -oP '\d+\.\d+' | head -1 || echo "0")

        cat > "$REPORT_DIR/coverage.json" << EOF
{
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "project": "SMART-QSO",
    "metrics": {
        "line_coverage": $LINE_COV,
        "function_coverage": $FUNC_COV,
        "branch_coverage": $BRANCH_COV
    },
    "thresholds": {
        "line": $LINE_THRESHOLD,
        "function": $FUNCTION_THRESHOLD,
        "branch": $BRANCH_THRESHOLD
    }
}
EOF
        echo -e "${GREEN}CI export: $REPORT_DIR/coverage.json${NC}"
    fi
}

# Main execution
case "${1:-html}" in
    --clean)
        clean_coverage
        ;;
    --summary)
        check_tools
        build_with_coverage
        run_tests
        collect_coverage
        generate_summary
        ;;
    --lcov)
        check_tools
        build_with_coverage
        run_tests
        collect_coverage
        generate_summary
        export_ci
        ;;
    --html|*)
        check_tools
        build_with_coverage
        run_tests
        collect_coverage
        generate_html
        generate_file_report
        generate_summary
        export_ci
        ;;
esac

echo ""
echo -e "${GREEN}Coverage generation complete!${NC}"
