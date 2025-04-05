#!/bin/bash
set -e

echo "Cleanning up previous builds..."
cargo clean

echo "Building debug version of LruOrderedMap tests..."
cd "$(dirname "$0")"
RUSTFLAGS="-C debuginfo=2" cargo test -p lru_ordered_map --test memory_leak_test --no-run

echo ""
echo "======================================================="
echo "Running memory leak tests with Valgrind..."
echo "======================================================="
echo ""

# Find the compiled test binary
TEST_BIN=$(find ../target/debug/deps -name "memory_leak_test-*" -type f -executable | head -1)

if [ -z "$TEST_BIN" ]; then
    echo "Error: Could not find test binary."
    exit 1
fi

echo "Found test binary: $TEST_BIN"
echo ""

# Run with Valgrind
valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes $TEST_BIN

echo ""
echo "======================================================="
echo "Valgrind memory leak test completed"
echo "======================================================="