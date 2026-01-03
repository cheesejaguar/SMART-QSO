#!/bin/bash
#
# SMART-QSO Flight Software Cyclomatic Complexity Checker
# Document: SQSO-FSW-SCRIPT-004
# Reference: NPR 7150.2, MISRA C:2012
#
# Usage: ./check_complexity.sh [--report|--warn|--strict]
#
# Thresholds (per NASA/JPL guidelines):
#   - CCN <= 10: Good
#   - CCN <= 15: Acceptable (warning)
#   - CCN > 15:  Requires review
#   - CCN > 20:  Not acceptable for flight
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SRC_DIR="$PROJECT_DIR/src"
REPORT_DIR="$PROJECT_DIR/reports"

# Thresholds
CCN_GOOD=10
CCN_WARN=15
CCN_FAIL=20
MAX_LENGTH=200
MAX_PARAMS=6
MAX_NESTING=4

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "=========================================="
echo "SMART-QSO Cyclomatic Complexity Check"
echo "=========================================="
echo "Source: $SRC_DIR"
echo ""

# Create report directory
mkdir -p "$REPORT_DIR"

# Check for lizard
check_lizard() {
    if ! command -v lizard &> /dev/null; then
        echo -e "${RED}Error: lizard not installed${NC}"
        echo "Install with: pip install lizard"
        exit 1
    fi
}

# Run lizard analysis
run_analysis() {
    local mode="$1"
    local threshold="$CCN_WARN"

    case "$mode" in
        --strict)
            threshold="$CCN_GOOD"
            ;;
        --warn)
            threshold="$CCN_WARN"
            ;;
        *)
            threshold="$CCN_FAIL"
            ;;
    esac

    echo -e "${BLUE}Running complexity analysis (threshold: $threshold)...${NC}"
    echo ""

    # Full report
    lizard \
        --CCN "$CCN_FAIL" \
        --length "$MAX_LENGTH" \
        --arguments "$MAX_PARAMS" \
        "$SRC_DIR" > "$REPORT_DIR/complexity_full.txt" 2>&1 || true

    # CSV for processing
    lizard \
        --csv \
        "$SRC_DIR" > "$REPORT_DIR/complexity.csv" 2>&1 || true

    # Warnings only
    lizard \
        --CCN "$threshold" \
        --length "$MAX_LENGTH" \
        --arguments "$MAX_PARAMS" \
        --warnings_only \
        "$SRC_DIR" > "$REPORT_DIR/complexity_warnings.txt" 2>&1 || true
}

# Analyze results
analyze_results() {
    echo -e "${BLUE}Complexity Analysis Results${NC}"
    echo "=========================================="

    # Parse CSV and categorize
    local good=0
    local warn=0
    local fail=0
    local total=0

    if [ -f "$REPORT_DIR/complexity.csv" ]; then
        # Skip header, process each function
        tail -n +2 "$REPORT_DIR/complexity.csv" | while IFS=, read -r nloc ccn token param length location file function_name etc; do
            # Clean up CCN value
            ccn_val=$(echo "$ccn" | tr -d '[:space:]"')

            if [ -n "$ccn_val" ] && [ "$ccn_val" -eq "$ccn_val" ] 2>/dev/null; then
                ((total++)) || true

                if [ "$ccn_val" -le "$CCN_GOOD" ]; then
                    ((good++)) || true
                elif [ "$ccn_val" -le "$CCN_WARN" ]; then
                    ((warn++)) || true
                else
                    ((fail++)) || true
                    echo -e "${RED}  HIGH: $function_name (CCN=$ccn_val) in $file${NC}"
                fi
            fi
        done
    fi

    # Summary from warnings file
    echo ""
    echo "Functions exceeding thresholds:"
    echo "-------------------------------"

    if [ -s "$REPORT_DIR/complexity_warnings.txt" ]; then
        # Parse lizard output format
        grep -E "^\s+[0-9]+" "$REPORT_DIR/complexity_warnings.txt" | while read -r line; do
            echo "  $line"
        done

        VIOLATION_COUNT=$(grep -cE "^\s+[0-9]+" "$REPORT_DIR/complexity_warnings.txt" 2>/dev/null || echo "0")
    else
        echo "  None - all functions within limits"
        VIOLATION_COUNT=0
    fi

    echo ""
    echo "=========================================="
    echo "Summary:"
    echo "  Total functions analyzed: $(wc -l < "$REPORT_DIR/complexity.csv" 2>/dev/null || echo "0")"
    echo "  Functions exceeding CCN $CCN_WARN: $VIOLATION_COUNT"
    echo ""
    echo "Thresholds:"
    echo "  Good (CCN <= $CCN_GOOD):     Ideal"
    echo "  Acceptable (CCN <= $CCN_WARN): May require review"
    echo "  High (CCN > $CCN_WARN):        Requires refactoring"
    echo "  Critical (CCN > $CCN_FAIL):    Not flight-acceptable"
}

# Generate detailed report
generate_report() {
    REPORT="$REPORT_DIR/complexity_report.md"

    cat > "$REPORT" << 'EOF'
# Cyclomatic Complexity Report
## SMART-QSO Flight Software

**Generated:** $(date)
**Reference:** NPR 7150.2, MISRA C:2012

---

## Thresholds

| Level | CCN | Action |
|-------|-----|--------|
| Good | <= 10 | None required |
| Acceptable | <= 15 | Consider refactoring |
| High | <= 20 | Refactoring required |
| Critical | > 20 | Not flight-acceptable |

---

## Results

EOF

    # Add full results
    if [ -f "$REPORT_DIR/complexity_full.txt" ]; then
        echo '```' >> "$REPORT"
        cat "$REPORT_DIR/complexity_full.txt" >> "$REPORT"
        echo '```' >> "$REPORT"
    fi

    echo ""
    echo -e "${GREEN}Report generated: $REPORT${NC}"
}

# Function-level recommendations
generate_recommendations() {
    echo ""
    echo -e "${BLUE}Refactoring Recommendations${NC}"
    echo "=========================================="

    if [ -s "$REPORT_DIR/complexity_warnings.txt" ]; then
        echo ""
        echo "For functions with high complexity:"
        echo "  1. Extract sub-functions for distinct logic blocks"
        echo "  2. Replace nested conditionals with early returns"
        echo "  3. Use lookup tables instead of switch statements"
        echo "  4. Apply state machine patterns for complex flows"
        echo "  5. Consider table-driven approaches"
        echo ""
        echo "See docs/development/CODING_STANDARDS.md for guidelines."
    else
        echo ""
        echo -e "${GREEN}All functions are within acceptable complexity limits.${NC}"
    fi
}

# Main execution
main() {
    local mode="${1:---report}"

    check_lizard
    run_analysis "$mode"
    analyze_results

    case "$mode" in
        --report)
            generate_report
            generate_recommendations
            ;;
        --strict)
            if [ -s "$REPORT_DIR/complexity_warnings.txt" ]; then
                echo -e "${RED}STRICT MODE: Complexity violations found${NC}"
                exit 1
            fi
            ;;
        --warn)
            if [ -s "$REPORT_DIR/complexity_warnings.txt" ]; then
                echo -e "${YELLOW}WARNING: Complexity threshold exceeded${NC}"
            fi
            ;;
    esac

    echo ""
    echo "Reports saved to: $REPORT_DIR/"
}

main "$@"
