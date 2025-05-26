#!/bin/bash

# Streaming Engine Performance Benchmark Runner
# This script runs all performance benchmarks for the streaming engine

set -e

echo "🚀 Starting Streaming Engine Performance Benchmarks"
echo "=================================================="

# Change to the streaming engine directory
cd "$(dirname "$0")/../streaming-engine"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Could not find Cargo.toml. Make sure you're running this from the project root."
    exit 1
fi

# Function to run a benchmark with nice output
run_benchmark() {
    local bench_name=$1
    local description=$2

    echo ""
    echo "📊 Running $description"
    echo "-------------------------------------------"

    if cargo bench --bench "$bench_name"; then
        echo "✅ $description completed successfully"
    else
        echo "❌ $description failed"
        return 1
    fi
}

# Function to run all benchmarks
run_all_benchmarks() {
    echo "🔧 Building project first..."
    cargo build --release

    echo ""
    echo "🏃 Running all benchmarks..."

    # Audio processing benchmarks
    run_benchmark "audio_processing" "Audio Processing Benchmarks"

    # Parameter parsing benchmarks
    run_benchmark "params_parsing" "Parameter Parsing Benchmarks"

    # Hash operations benchmarks
    run_benchmark "hash_operations" "Hash Operations Benchmarks"

    # Storage operations benchmarks
    run_benchmark "storage_operations" "Storage Operations Benchmarks"

    # End-to-end streaming engine benchmarks
    run_benchmark "streaming_engine" "End-to-End Streaming Engine Benchmarks"
}

# Function to run a specific benchmark
run_specific_benchmark() {
    local bench_name=$1

    case "$bench_name" in
        "audio"|"audio_processing")
            run_benchmark "audio_processing" "Audio Processing Benchmarks"
            ;;
        "params"|"params_parsing")
            run_benchmark "params_parsing" "Parameter Parsing Benchmarks"
            ;;
        "hash"|"hash_operations")
            run_benchmark "hash_operations" "Hash Operations Benchmarks"
            ;;
        "storage"|"storage_operations")
            run_benchmark "storage_operations" "Storage Operations Benchmarks"
            ;;
        "engine"|"streaming_engine")
            run_benchmark "streaming_engine" "End-to-End Streaming Engine Benchmarks"
            ;;
        *)
            echo "❌ Unknown benchmark: $bench_name"
            echo "Available benchmarks:"
            echo "  - audio (audio_processing)"
            echo "  - params (params_parsing)"
            echo "  - hash (hash_operations)"
            echo "  - storage (storage_operations)"
            echo "  - engine (streaming_engine)"
            exit 1
            ;;
    esac
}

# Function to show benchmark results comparison
compare_results() {
    echo ""
    echo "📈 Benchmark Results Summary"
    echo "============================"

    # Look for recent benchmark results
    if command -v divan &> /dev/null; then
        echo "💡 Use 'cargo bench -- --help' for advanced result analysis"
    fi

    echo ""
    echo "📝 Benchmark files location:"
    echo "   - Audio processing: benches/audio_processing.rs"
    echo "   - Params parsing: benches/params_parsing.rs"
    echo "   - Hash operations: benches/hash_operations.rs"
    echo "   - Storage operations: benches/storage_operations.rs"
    echo "   - End-to-end engine: benches/streaming_engine.rs"
}

# Function to show help
show_help() {
    echo "Streaming Engine Benchmark Runner"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  all                 Run all benchmarks (default)"
    echo "  audio              Run audio processing benchmarks only"
    echo "  params             Run parameter parsing benchmarks only"
    echo "  hash               Run hash operations benchmarks only"
    echo "  storage            Run storage operations benchmarks only"
    echo "  engine             Run end-to-end streaming engine benchmarks only"
    echo "  compare            Show benchmark results comparison"
    echo "  help               Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                 # Run all benchmarks"
    echo "  $0 audio           # Run only audio processing benchmarks"
    echo "  $0 engine          # Run only end-to-end benchmarks"
    echo "  $0 compare         # Show results summary"
    echo ""
    echo "Environment Variables:"
    echo "  BENCH_ARGS         Additional arguments to pass to cargo bench"
    echo "                     Example: BENCH_ARGS='--verbose' $0"
}

# Main script logic
main() {
    local command=${1:-all}

    case "$command" in
        "all"|"")
            run_all_benchmarks
            compare_results
            ;;
        "compare")
            compare_results
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            run_specific_benchmark "$command"
            ;;
    esac

    echo ""
    echo "🎉 Benchmark run completed!"
    echo ""
    echo "💡 Tips:"
    echo "   - Run benchmarks multiple times for consistent results"
    echo "   - Close other applications to reduce system noise"
    echo "   - Use 'cargo bench --bench <name>' for specific benchmarks"
    echo "   - Results are automatically cached by divan for comparison"
}

# Handle Ctrl+C gracefully
trap 'echo -e "\n⚠️  Benchmark run interrupted by user"; exit 130' INT

# Run main function with all arguments
main "$@"
