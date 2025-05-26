#!/bin/bash

# CodSpeed Integration Verification Script
# This script tests that CodSpeed is properly configured for our benchmarks

set -e

echo "🔧 CodSpeed Integration Verification"
echo "====================================="

# Check if we're in the right directory
if [ ! -f "streaming-engine/Cargo.toml" ]; then
    echo "❌ Error: Must be run from the project root directory"
    exit 1
fi

# Check if cargo-codspeed is installed
if ! command -v cargo-codspeed &> /dev/null; then
    echo "📦 Installing cargo-codspeed..."
    cargo install cargo-codspeed --locked
else
    echo "✅ cargo-codspeed is already installed"
fi

# Navigate to streaming-engine directory
cd streaming-engine

echo ""
echo "🏗️ Building benchmarks with CodSpeed..."
if cargo codspeed build; then
    echo "✅ CodSpeed build successful"
else
    echo "❌ CodSpeed build failed"
    exit 1
fi

echo ""
echo "🧪 Testing CodSpeed run (dry run)..."
if cargo codspeed run 2>/dev/null | grep -q "Checked:"; then
    echo "✅ CodSpeed run test successful"
else
    echo "❌ CodSpeed run test failed"
    exit 1
fi

echo ""
echo "📊 Verifying benchmark structure..."

# Check that all benchmark files exist
BENCHMARKS=("audio_processing" "params_parsing" "hash_operations" "storage_operations" "streaming_engine")
for bench in "${BENCHMARKS[@]}"; do
    if [ -f "benches/${bench}.rs" ]; then
        echo "✅ Benchmark found: ${bench}.rs"
    else
        echo "❌ Missing benchmark: ${bench}.rs"
        exit 1
    fi
done

echo ""
echo "🔍 Verifying Cargo.toml configuration..."

# Check CodSpeed divan compatibility
if grep -q "codspeed-divan-compat" Cargo.toml; then
    echo "✅ CodSpeed Divan compatibility layer configured"
else
    echo "❌ CodSpeed Divan compatibility layer not found in Cargo.toml"
    exit 1
fi

# Check benchmark harness configuration
if grep -q "harness = false" Cargo.toml; then
    echo "✅ Benchmark harness properly disabled"
else
    echo "❌ Benchmark harness configuration missing"
    exit 1
fi

echo ""
echo "⚡ Testing individual benchmark builds..."

for bench in "${BENCHMARKS[@]}"; do
    echo -n "  Testing ${bench}... "
    if cargo codspeed build "${bench}" &>/dev/null; then
        echo "✅"
    else
        echo "❌"
        echo "    Failed to build ${bench} benchmark"
        exit 1
    fi
done

echo ""
echo "🌍 Verifying GitHub Actions workflow..."
cd ..

if [ -f ".github/workflows/codspeed.yml" ]; then
    echo "✅ CodSpeed GitHub Actions workflow found"

    # Check key workflow components
    if grep -q "cargo codspeed build" .github/workflows/codspeed.yml; then
        echo "✅ Build step configured"
    else
        echo "⚠️  Warning: Build step might be missing in workflow"
    fi

    if grep -q "CodSpeedHQ/action@v3" .github/workflows/codspeed.yml; then
        echo "✅ CodSpeed action configured"
    else
        echo "⚠️  Warning: CodSpeed action might be missing in workflow"
    fi

    if grep -q "CODSPEED_TOKEN" .github/workflows/codspeed.yml; then
        echo "✅ CodSpeed token configured"
    else
        echo "⚠️  Warning: CodSpeed token configuration missing"
    fi
else
    echo "❌ CodSpeed GitHub Actions workflow not found"
    echo "    Expected: .github/workflows/codspeed.yml"
    exit 1
fi

echo ""
echo "📝 Verifying documentation..."

if [ -f "BENCHMARKS.md" ]; then
    if grep -q "CodSpeed" BENCHMARKS.md; then
        echo "✅ CodSpeed documentation found in BENCHMARKS.md"
    else
        echo "⚠️  Warning: CodSpeed documentation might be incomplete"
    fi
else
    echo "⚠️  Warning: BENCHMARKS.md not found"
fi

echo ""
echo "🎉 CodSpeed Integration Verification Complete!"
echo ""
echo "Summary:"
echo "✅ cargo-codspeed CLI installed and working"
echo "✅ All benchmark files present and building"
echo "✅ CodSpeed compatibility layer configured"
echo "✅ GitHub Actions workflow configured"
echo "✅ Documentation updated"
echo ""
echo "Next steps:"
echo "1. Ensure CODSPEED_TOKEN secret is set in GitHub repository settings"
echo "2. Push changes to trigger first CodSpeed run"
echo "3. Monitor performance reports on pull requests"
echo ""
echo "Local testing commands:"
echo "  cargo codspeed build -p streaming-engine"
echo "  cargo codspeed run"
echo "  ./run_benchmarks.sh"
echo ""
echo "📊 CodSpeed Dashboard: https://codspeed.io/"
