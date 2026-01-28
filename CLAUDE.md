# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SnowID is a high-performance distributed ID generator library (like Twitter's Snowflake) that generates unique 64-bit identifiers. IDs are time-sorted, monotonic, and thread-safe.

**Key characteristics:**
- 42-bit timestamp + configurable node bits (6-16) + sequence bits
- ~244ns per ID generation time
- Supports up to 65,536 nodes (with 16 node bits)
- Base62 encoding support for compact, URL-friendly IDs

## Common Development Commands

```bash
# Build the project
cargo build

# Build with optimizations (release profile uses LTO, single codegen unit, panic abort)
cargo build --release

# Run all tests
cargo test

# Run specific test module
cargo test core_tests
cargo test concurrent_tests

# Run tests with output
cargo test -- --nocapture

# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench snowid_benchmarks
cargo bench --bench base62_benchmarks

# Check code without building
cargo check

# Run lints
cargo clippy

# Format code
cargo fmt

# Run examples
cargo run --example basic
cargo run --example custom_config
cargo run --example distributed
```

## Architecture

The codebase follows clean architecture with performance-critical optimizations:

### Core Components

**`src/lib.rs`** - Main ID generator
- `SnowID` struct aligned to 64-byte cache lines to prevent false sharing
- Fast path/slow path optimization for ID generation
- Fast path: direct sequence increment (~244ns typical)
- Slow path: timestamp advancement with exponential backoff
- Uses `AtomicU64` for timestamp and `AtomicU16` for sequence (thread-safe)
- Bit manipulation with branchless operations for performance
- Base62 encoding/decoding utilities

**`src/config.rs`** - Configuration management
- Builder pattern for flexible setup
- Const-evaluable methods for compile-time optimization
- Configurable node bits (6-16), custom epoch, spin/yield behavior
- Copy-optimized `#[repr(C)]` struct
- Pre-computes bitmasks and shift values for performance

**`src/extractor.rs`** - Component extraction
- Decomposes IDs into timestamp, node, and sequence
- Single-pass extraction operations
- Uses same config as generator for consistency
- Inline methods for performance

**`src/error.rs`** - Error types
- `InvalidNodeId` - node ID exceeds max for config
- `ClockMovedBackwards` - system clock went backwards
- Clone-able errors for error chaining

### Key Design Patterns

**Fast Path/Slow Path:**
- Common case (same millisecond): atomic fetch_add on sequence
- Overflow case (sequence exhausted): wait for next millisecond with exponential backoff
- Spin/yield tuning available via config

**Bit Allocation (64-bit ID):**
- Bits 0-41: Timestamp (42 bits = 139 years from epoch)
- Bits 42-63: Node + Sequence (22 bits total, configurable split)
- Example: 10 node bits + 12 sequence bits = 1,024 nodes × 4,096 IDs/ms

**Memory Ordering:**
- Uses `Ordering::Relaxed` for sequence (same millisecond)
- Uses `Ordering::Acquire` for timestamp reads
- Uses `Ordering::Release` for timestamp writes
- Carefully chosen to balance performance and correctness

### Thread Safety

- No `unsafe` code (`#![forbid(unsafe_code)]`)
- Atomics ensure thread-safe ID generation
- Multiple generators can share same node ID (but may cause sequence overflow more frequently)
- Cache-line alignment prevents false sharing between generators

### Testing Structure

Tests are organized in `src/tests/`:
- `core_tests.rs` - basic generation and configuration
- `concurrent_tests.rs` - multi-threaded generation
- `sequence_tests.rs` - sequence overflow and backoff behavior
- `timing_tests.rs` - monotonicity and timestamp handling
- `boundary_tests.rs` - edge cases and limits
- `base62_tests.rs` - Base62 encoding/decoding
- `extraction_tests.rs` - component decomposition

## Important Constraints

1. **Node bits must be between 6-16** (enforced by config builder)
2. **Node ID must be ≤ max_node_id** for the configuration
3. **System clock must not move backwards** significantly (will return error)
4. **Sequence overflow per millisecond** triggers backoff (normal behavior)
5. **No unsafe code allowed** - project forbids it

## Performance Considerations

- Use `cargo build --release` for realistic performance testing
- The fast path optimization is critical - most ID generations hit this
- Spin/yield tuning affects tail latency under high throughput
- Base62 encoding adds ~2% overhead vs raw u64
- Cache-line alignment is essential for multi-threaded performance

## Dependencies

Core dependencies:
- `thiserror` - error derive macros
- `base62` - Base62 encoding/decoding

Dev dependencies:
- `criterion` - benchmarking with HTML reports
- `rand` - randomized testing
- `chrono` - timestamp utilities in tests
