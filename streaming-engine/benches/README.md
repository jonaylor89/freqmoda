# Streaming Engine Performance Benchmarks

This directory contains comprehensive performance benchmarks for the streaming engine, designed to measure and monitor the performance of critical components.

## Overview

The benchmarks are built using [Divan](https://docs.rs/divan/latest/divan/), a modern Rust benchmarking framework that provides:
- Statistical analysis of performance data
- Comparison between benchmark runs
- Memory usage tracking
- Throughput measurements

## Benchmark Files

### 1. `audio_processing.rs`
Tests the core audio processing pipeline:
- **Audio Buffer Creation**: Tests creation of `AudioBuffer` from raw bytes for different formats (MP3, WAV, FLAC, OGG)
- **Format Detection**: Benchmarks automatic audio format detection from file headers
- **Audio Processing**: End-to-end audio processing with different complexity levels
- **Concurrent Processing**: Multi-threaded audio processing performance
- **Memory Operations**: Buffer cloning, conversion, and memory usage patterns

**Key Metrics:**
- Processing time per audio size (1KB - 1MB)
- Memory allocation patterns
- Concurrent processing throughput

### 2. `params_parsing.rs`
Benchmarks parameter parsing and URL handling:
- **Parameter Parsing**: From query strings and URL paths
- **Serialization**: Converting parameters to different formats (query strings, FFmpeg args)
- **URL Encoding/Decoding**: Performance of URL-safe string operations
- **Complex Parameter Handling**: Large parameter sets with multiple filters
- **Error Handling**: Performance impact of invalid input processing

**Key Metrics:**
- Parse time per parameter complexity
- Serialization overhead
- URL encoding/decoding throughput

### 3. `hash_operations.rs`
Tests cryptographic and hashing operations:
- **SHA1 Hashing**: Storage path hashing for different input sizes
- **Parameter Hashing**: Hash generation from audio processing parameters
- **Argon2 Operations**: Password hashing and verification
- **Hash Consistency**: Ensuring identical inputs produce identical hashes
- **Collision Resistance**: Testing hash uniqueness for similar inputs

**Key Metrics:**
- Hash computation time
- Throughput (MB/s) for different data sizes
- Memory usage during hashing

### 4. `storage_operations.rs`
Benchmarks file system and caching operations:
- **File Storage**: Put/get/delete operations with different file sizes
- **Cache Operations**: Memory and disk-based caching performance
- **Path Normalization**: Safe filename generation
- **Storage vs Cache**: Performance comparison between storage backends
- **Error Recovery**: Handling of missing files and cache misses

**Key Metrics:**
- I/O throughput for different file sizes
- Cache hit/miss performance
- Storage operation latency

### 5. `streaming_engine.rs`
End-to-end integration benchmarks:
- **Full Pipeline**: Complete audio processing workflow
- **Concurrent Requests**: Multi-user scenario simulation
- **Cache Performance**: Real-world cache hit ratio patterns
- **Memory Pressure**: Large file processing under memory constraints
- **Error Recovery**: System resilience under failure conditions

**Key Metrics:**
- End-to-end processing time
- System throughput under load
- Memory usage patterns
- Error recovery time

## Running Benchmarks

### Quick Start

Use the provided benchmark runner script:

```bash
# Run all benchmarks
./run_benchmarks.sh

# Run specific benchmark categories
./run_benchmarks.sh audio      # Audio processing only
./run_benchmarks.sh params     # Parameter parsing only
./run_benchmarks.sh hash       # Hash operations only
./run_benchmarks.sh storage    # Storage operations only
./run_benchmarks.sh engine     # End-to-end benchmarks only
```

### Manual Execution

Run individual benchmarks using Cargo:

```bash
cd streaming-engine

# Run specific benchmark
cargo bench --bench audio_processing
cargo bench --bench params_parsing  
cargo bench --bench hash_operations
cargo bench --bench storage_operations
cargo bench --bench streaming_engine

# Run all benchmarks
cargo bench
```

### Advanced Options

```bash
# Run with verbose output
cargo bench --bench audio_processing -- --verbose

# Run specific benchmark function
cargo bench --bench audio_processing -- audio_buffer_creation

# Save results for comparison
cargo bench > benchmark_results.txt
```

## Interpreting Results

### Understanding Output

Divan provides several key metrics:

- **fastest/slowest**: Min/max execution times observed
- **median**: Middle value, less affected by outliers
- **mean**: Average execution time
- **samples**: Number of benchmark iterations
- **iters**: Inner iterations per sample

### Example Output

```
audio_buffer_creation           fastest    │ slowest    │ median     │ mean       │ samples │ iters
├─ mp3_from_bytes                          │            │            │            │         │
│  ├─ 1                         504.8 ns   │ 517.8 ns   │ 512.7 ns   │ 510.2 ns   │ 100     │ 1600
│  ├─ 10                        4.915 µs   │ 5.082 µs   │ 4.957 µs   │ 4.965 µs   │ 100     │ 100
│  └─ 100                       49.12 µs   │ 50.99 µs   │ 49.2 µs    │ 49.24 µs   │ 100     │ 100
```

### Performance Targets

#### Audio Processing
- Small files (1-10KB): < 10ms processing time
- Medium files (100KB): < 100ms processing time  
- Large files (1MB+): < 1s processing time

#### Parameter Operations
- Simple parsing: < 1µs
- Complex parameter sets: < 10µs
- URL encoding/decoding: < 1µs

#### Hash Operations
- SHA1 hashing: > 100 MB/s throughput
- Parameter hashing: < 10µs
- Argon2 operations: 10-50ms (security vs performance trade-off)

#### Storage Operations
- Cache hits: < 1ms
- File I/O: 50-200 MB/s depending on size
- Path normalization: < 1µs

## Benchmarking Best Practices

### Environment Setup

For consistent results:

1. **Close unnecessary applications** to reduce system noise
2. **Run on AC power** (not battery) for laptops
3. **Disable CPU frequency scaling** if possible
4. **Run multiple times** and compare median values
5. **Use dedicated benchmark machine** for CI/CD

### Interpreting Variance

- **Low variance** (< 5%): Reliable, consistent performance
- **Medium variance** (5-20%): Acceptable for most use cases  
- **High variance** (> 20%): Investigate for bottlenecks or system interference

### Regression Detection

Monitor these key indicators:
- **Significant slowdown** (> 20% increase in median time)
- **Memory usage growth** (check for leaks)
- **Throughput degradation** (MB/s decreases)
- **Increased variance** (less predictable performance)

## Development Workflow

### Adding New Benchmarks

1. **Identify bottleneck**: Profile code to find performance-critical sections
2. **Create focused benchmark**: Test specific functionality, not entire systems
3. **Use realistic data**: Mirror production workloads and data sizes
4. **Add multiple scenarios**: Test edge cases and different input sizes
5. **Document expectations**: Include performance targets and reasoning

### Benchmark Hygiene

```rust
use divan::{Bencher, black_box};

#[divan::bench]
fn benchmark_function() {
    let input = create_test_data();
    
    // Use black_box to prevent compiler optimizations
    let result = process_data(black_box(input));
    black_box(result);
}
```

### CI Integration

Add to CI pipeline:
```yaml
- name: Run Benchmarks
  run: |
    cd streaming-engine
    cargo bench --bench audio_processing -- --output-format json > bench_results.json
```

## Troubleshooting

### Common Issues

**FFmpeg Errors in Output**: 
- Expected when using mock audio data
- Doesn't affect benchmark timing accuracy
- Can be filtered with `2>/dev/null`

**Inconsistent Results**:
- Check system load and other processes
- Verify consistent input data
- Run longer benchmark cycles

**Memory Issues**:
- Monitor system memory during benchmarks
- Use smaller test data sizes for memory-constrained systems
- Check for memory leaks in long-running benchmarks

### Performance Debugging

1. **Profile first**: Use `cargo flamegraph` or similar tools
2. **Isolate components**: Test individual functions before integration
3. **Check algorithms**: Ensure O(n) complexity expectations
4. **Monitor resources**: CPU, memory, I/O during benchmarks

## Contributing

When adding benchmarks:

1. **Follow naming conventions**: `benchmark_group::specific_function`
2. **Include documentation**: Explain what's being measured
3. **Add performance targets**: Expected ranges for pass/fail
4. **Test on multiple platforms**: Ensure cross-platform compatibility
5. **Update this README**: Document new benchmark categories

## Resources

- [Divan Documentation](https://docs.rs/divan/latest/divan/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion.rs Comparison](https://bheisler.github.io/criterion.rs/book/)
- [Streaming Engine Architecture](../README.md)