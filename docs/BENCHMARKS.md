# Streaming Engine Performance Benchmarks

This document provides an overview of the comprehensive performance benchmarking suite for the FreqModa streaming engine.

## Overview

The benchmarking suite consists of 5 specialized benchmark modules designed to measure and monitor the performance of critical components in the audio streaming pipeline. All benchmarks are built using [Divan](https://docs.rs/divan/latest/divan/), a modern Rust benchmarking framework.

## Benchmark Modules

### 1. Audio Processing (`audio_processing.rs`)
- **Audio Buffer Creation**: Performance of creating AudioBuffer instances from raw bytes
- **Format Detection**: Speed of automatic audio format detection (MP3, WAV, FLAC, OGG)
- **Audio Processing Pipeline**: End-to-end processing with various complexity levels
- **Concurrent Processing**: Multi-threaded audio processing performance
- **Memory Operations**: Buffer operations, cloning, and memory usage patterns

### 2. Parameter Parsing (`params_parsing.rs`)
- **URL Parameter Parsing**: Performance of parsing query strings and URL paths
- **Parameter Serialization**: Converting parameters to different output formats
- **URL Encoding/Decoding**: String encoding operations for web safety
- **Complex Parameter Handling**: Processing large parameter sets with multiple filters
- **Error Handling**: Performance impact of processing invalid inputs

### 3. Hash Operations (`hash_operations.rs`)
- **SHA1 Hashing**: Storage path hashing for different input sizes
- **Parameter Hashing**: Hash generation from audio processing parameters
- **Argon2 Operations**: Secure password hashing and verification
- **Hash Consistency**: Ensuring deterministic hash generation
- **Collision Resistance**: Testing hash uniqueness for similar inputs

### 4. Storage Operations (`storage_operations.rs`)
- **File Storage Operations**: Put/get/delete operations across file sizes
- **Cache Performance**: Memory and disk-based caching benchmarks
- **Path Normalization**: Safe filename generation and path handling
- **Storage Backend Comparison**: Performance differences between storage types
- **Error Recovery**: Handling missing files and cache misses

### 5. End-to-End Pipeline (`streaming_engine.rs`)
- **Full Processing Pipeline**: Complete audio streaming workflow
- **Concurrent Request Handling**: Multi-user scenario simulation
- **Cache Hit Ratio Patterns**: Real-world cache performance modeling
- **Memory Pressure Testing**: Large file processing under constraints
- **System Resilience**: Error recovery and failure handling

## Quick Start

### Using the Benchmark Runner

```bash
# Run all benchmarks
./run_benchmarks.sh

# Run specific categories
./run_benchmarks.sh audio      # Audio processing only
./run_benchmarks.sh params     # Parameter parsing only
./run_benchmarks.sh hash       # Hash operations only
./run_benchmarks.sh storage    # Storage operations only
./run_benchmarks.sh engine     # End-to-end benchmarks only

# Get help
./run_benchmarks.sh help
```

### Manual Execution

```bash
cd streaming-engine

# Run individual benchmarks
cargo bench --bench audio_processing
cargo bench --bench params_parsing
cargo bench --bench hash_operations
cargo bench --bench storage_operations
cargo bench --bench streaming_engine

# Run all benchmarks
cargo bench
```

### CodSpeed Integration

Our benchmarks are configured to work with [CodSpeed](https://codspeed.io/) for continuous performance monitoring:

```bash
# Install CodSpeed CLI (one-time setup)
cargo install cargo-codspeed --locked

# Build benchmarks for CodSpeed
cargo codspeed build -p streaming-engine

# Run benchmarks locally (for testing)
cargo codspeed run

# Run specific benchmark
cargo codspeed run audio_processing
```

The benchmarks automatically run on every push and pull request through GitHub Actions, providing:
- Performance regression detection
- Historical performance tracking
- Automated performance reports on PRs
- Continuous monitoring across different environments

## Performance Targets

### Audio Processing
- Small files (1-10KB): < 10ms
- Medium files (100KB): < 100ms
- Large files (1MB+): < 1s

### Parameter Operations
- Simple parsing: < 1µs
- Complex parameter sets: < 10µs
- URL encoding/decoding: < 1µs

### Hash Operations
- SHA1 throughput: > 100 MB/s
- Parameter hashing: < 10µs
- Argon2 operations: 10-50ms

### Storage Operations
- Cache hits: < 1ms
- File I/O: 50-200 MB/s
- Path normalization: < 1µs

## Sample Results

```
audio_processing                  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ audio_buffer_creation                        │               │               │               │         │
│  ├─ mp3_from_bytes                           │               │               │               │         │
│  │  ├─ 1                        504.8 ns      │ 517.8 ns      │ 512.7 ns      │ 510.2 ns      │ 100     │ 1600
│  │  ├─ 10                       4.915 µs      │ 5.082 µs      │ 4.957 µs      │ 4.965 µs      │ 100     │ 100
│  │  └─ 100                      49.12 µs      │ 50.99 µs      │ 49.2 µs       │ 49.24 µs      │ 100     │ 100

hash_operations                   fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ sha1_hashing                                 │               │               │               │         │
│  ├─ digest_storage_hasher_short  225.9 ns     │ 309.2 ns      │ 231.1 ns      │ 233.1 ns      │ 100     │ 1600
│  │                              35.4 MB/s     │ 25.86 MB/s    │ 34.6 MB/s     │ 34.31 MB/s    │         │
│  └─ digest_storage_hasher_long   288.4 ns     │ 403 ns        │ 293.6 ns      │ 297.8 ns      │ 100     │ 1600
│                                  197.5 MB/s   │ 141.4 MB/s    │ 194 MB/s      │ 191.4 MB/s    │         │
```

## Best Practices

### Environment Setup
1. Close unnecessary applications to reduce system noise
2. Run on AC power for laptops
3. Disable CPU frequency scaling if possible
4. Run multiple times and compare median values
5. Use dedicated benchmark machine for CI/CD

### Interpreting Results
- **Low variance** (< 5%): Reliable, consistent performance
- **Medium variance** (5-20%): Acceptable for most use cases
- **High variance** (> 20%): Investigate for bottlenecks

### Regression Detection
Monitor for:
- Significant slowdown (> 20% increase in median time)
- Memory usage growth
- Throughput degradation
- Increased variance

## Files Structure

```
streaming-engine/
├── benches/
│   ├── README.md                    # Detailed benchmark documentation
│   ├── audio_processing.rs          # Audio pipeline benchmarks
│   ├── params_parsing.rs           # Parameter handling benchmarks
│   ├── hash_operations.rs          # Cryptographic operation benchmarks
│   ├── storage_operations.rs       # File system and cache benchmarks
│   └── streaming_engine.rs         # End-to-end integration benchmarks
├── Cargo.toml                      # Benchmark configuration with CodSpeed
└── run_benchmarks.sh               # Benchmark runner script
├── .github/workflows/codspeed.yml  # CodSpeed CI configuration
└── BENCHMARKS.md                   # This documentation
```

### CodSpeed Configuration

The benchmarks use the CodSpeed compatibility layer for Divan

This configuration:
- Maintains full compatibility with standard Divan benchmarks
- Enables CodSpeed instrumentation when running in CI
- Provides performance regression detection
- Generates detailed performance reports

## CI Integration

### GitHub Actions with CodSpeed

Our benchmarks are automatically integrated with CodSpeed for continuous performance monitoring. The workflow (`.github/workflows/codspeed.yml`) runs on:

- Every push to `main` branch
- Every pull request
- Manual workflow dispatch

```yaml
name: CodSpeed

on:
  push:
    branches: ["main"]
  pull_request:
  workflow_dispatch:

jobs:
  benchmarks:
    name: Run benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup rust toolchain, cache and cargo-codspeed binary
        uses: moonrepo/setup-rust@v1
        with:
          channel: stable
          cache-target: release
          bins: cargo-codspeed
      - name: Install FFmpeg (required for audio processing)
        run: |
          sudo apt-get update
          sudo apt-get install -y ffmpeg
      - name: Build the benchmark target(s)
        run: cargo codspeed build -p streaming-engine
      - name: Run the benchmarks
        uses: CodSpeedHQ/action@v3
        with:
          run: cargo codspeed run
          token: ${{ secrets.CODSPEED_TOKEN }}
```

### Custom CI Integration

For other CI systems, you can run benchmarks manually:

```bash
# Traditional benchmarking
cd streaming-engine
cargo bench --bench audio_processing -- --output-format json > audio_bench.json
cargo bench --bench hash_operations -- --output-format json > hash_bench.json
# Compare with baseline and fail if regression detected
```

## Troubleshooting

### Common Issues

- **FFmpeg Errors**: Expected when using mock audio data; doesn't affect timing
- **Inconsistent Results**: Check system load and other processes
- **Memory Issues**: Monitor system memory and use smaller test data if needed

### Performance Debugging

1. Profile first using `cargo flamegraph`
2. Isolate components before integration testing
3. Verify algorithmic complexity expectations
4. Monitor CPU, memory, and I/O during benchmarks

## Contributing

When adding new benchmarks:

1. Follow naming conventions: `benchmark_group::specific_function`
2. Include documentation explaining what's being measured
3. Add performance targets and expected ranges
4. Test on multiple platforms
5. Update documentation
6. Verify CodSpeed compatibility by running `cargo codspeed build` locally

### CodSpeed Setup for Contributors

1. **Install CodSpeed CLI**: `cargo install cargo-codspeed --locked`
2. **Test locally**: `cargo codspeed build -p streaming-engine && cargo codspeed run`
3. **Verify CI integration**: Ensure benchmarks run successfully in GitHub Actions
4. **Monitor performance**: Check CodSpeed reports on pull requests

## Resources

- [Benchmark Details](streaming-engine/benches/README.md)
- [Divan Documentation](https://docs.rs/divan/latest/divan/)
- [CodSpeed Documentation](https://docs.codspeed.io/benchmarks/rust/divan)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Project Architecture](README.md)
- [CodSpeed Dashboard](https://codspeed.io/) (for viewing performance results)
