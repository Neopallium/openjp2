# OpenJPEG Rust Integration Tests

This directory contains integration tests for the OpenJPEG tools, ported from the original CTest suite.

## Overview

The tests use the CLI library functions directly instead of spawning subprocesses:
- `run_compress()` - Compress images to JPEG 2000
- `run_decompress()` - Decompress JPEG 2000 to images
- `run_dump()` - Dump codestream information
- `run_compare_images()` - Compare two images
- `run_compare_dump_files()` - Compare dump file outputs
- `run_compare_raw_files()` - Compare raw files

## Test Structure

```
tests/
├── common/
│   └── mod.rs          # Shared test utilities (MD5, paths, etc.)
└── README.md           # This file
```

## Writing Tests

### Basic Pattern

```rust
#[test]
fn test_example() {
}
```

### Using the `args!` Macro

The `args!` macro helps build argument vectors:

```rust
// Instead of:
let args = vec![
    String::from("test"),
    String::from("-i"),
    String::from("input.ppm"),
];

// Use:
let args = args!["-i", "input.ppm"];
```

### Common Test Utilities

#### Path Helpers
- `get_test_data_root()` - Root test data directory
- `get_input_dir()` - Input test files (`data/input/`)
- `get_baseline_dir()` - Baseline reference files (`data/baseline/`)
- `get_temp_dir()` - Temporary output directory

#### File Comparison
- `md5_file(path)` - Compute MD5 hash of a file
- `files_match_md5(file1, file2)` - Compare two files by MD5

## Running Tests

```bash
# Run all tests
cd openjp2-rs/tools
cargo test

# Run specific test file
cargo test --test nonreg_encode

# Run with output
cargo test -- --nocapture

# Set test data location
OPJ_DATA_ROOT=/path/to/data cargo test
```

## Test Organization

- **nonreg_encode.rs** - Non-regression encode tests (NR-ENC-*)
- **nonreg_decode.rs** - Non-regression decode tests (NR-DEC-*)
- **conformance.rs** - Conformance tests
- **dump.rs** - Dump tool tests
- **compare.rs** - Comparison tool tests
