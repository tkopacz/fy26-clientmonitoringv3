#!/bin/bash
# Comprehensive test runner for Monitoring System (Rust agent + .NET server)
# 
# This script runs both Rust and .NET tests in the correct order:
# 1. Rust agent tests (includes cross-language serialization test that generates test data)
# 2. .NET server tests (includes cross-language deserialization test that validates the data)
#
# Usage: ./run-all-tests.sh [options]
# Options:
#   --verbose   - Show full test output
#   --help      - Show this help message

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            cat "$0"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${YELLOW}=== Monitoring System Test Suite ===${NC}"
echo "Running tests in: $SCRIPT_DIR"
echo ""

# Track test results
RUST_FAILED=false
DOTNET_FAILED=false

# Run Rust tests
echo -e "${YELLOW}[1/2] Running Rust agent tests...${NC}"
if cd "$SCRIPT_DIR" && cargo test --package agent; then
    echo -e "${GREEN}✓ Rust tests passed${NC}"
else
    echo -e "${RED}✗ Rust tests failed${NC}"
    RUST_FAILED=true
fi
echo ""

# Run .NET tests
echo -e "${YELLOW}[2/2] Running .NET server tests...${NC}"
if cd "$SCRIPT_DIR/server" && dotnet test Tests/MonitoringServer.Tests.csproj \
    $(if [ "$VERBOSE" = false ]; then echo "--verbosity minimal"; else echo "--verbosity normal"; fi); then
    echo -e "${GREEN}✓ .NET tests passed${NC}"
else
    echo -e "${RED}✗ .NET tests failed${NC}"
    DOTNET_FAILED=true
fi
echo ""

# Summary
echo -e "${YELLOW}=== Test Summary ===${NC}"
if [ "$RUST_FAILED" = false ] && [ "$DOTNET_FAILED" = false ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    exit 0
else
    if [ "$RUST_FAILED" = true ]; then
        echo -e "${RED}✗ Rust agent tests failed${NC}"
    fi
    if [ "$DOTNET_FAILED" = true ]; then
        echo -e "${RED}✗ .NET server tests failed${NC}"
    fi
    exit 1
fi
