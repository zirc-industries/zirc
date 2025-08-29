# Zirc Language Benchmarks

This directory contains performance benchmarks for the Zirc language interpreter and toolchain.

## Structure

```
benchmark/
├── README.md           # This file
├── scripts/            # Benchmark test scripts (auto-discovered)
│   ├── fibonacci.zirc
│   ├── factorial.zirc
│   ├── sorting.zirc
│   └── loops.zirc
└── results/            # Benchmark output results (JSON)

crates/
└── benchmark-runner/   # Binary that runs benchmarks (package: zirc-bench)
```

## Running Benchmarks

### Prerequisites
- Build the workspace (first run will fetch dependencies):
  - `cargo build --release`

### Execute Benchmarks
```
# Run all benchmarks (default: only benchmark/scripts, output silenced)
cargo run -p zirc-bench --release

# Include examples/ in discovery (may include interactive scripts)
cargo run -p zirc-bench --release -- --include-examples

# Run specific benchmark(s)
cargo run -p zirc-bench --release -- --test fibonacci --test loops

# Run with custom iterations / warmup and custom output path
cargo run -p zirc-bench --release -- --iterations 100 --warmup 5 --output benchmark/results/custom.json

# Disable silencing to see program prints (not recommended for timing)
cargo run -p zirc-bench --release -- --silent=false

# List available tests without running them
cargo run -p zirc-bench --release -- --list
```

## What is measured?
For each test we measure (in milliseconds):
- Total time (lex + parse + exec)
- Average lex, parse, exec times
- Min / Max total times
- Interpreter memory usage (KB) observed after execution

## Adding New Benchmarks

1. Create a new `.zirc` file in `benchmark/scripts/` (or reuse a file in `examples/`)
2. Re-run the runner — files are auto-discovered by filename (name is the file stem)
3. Keep I/O inside tight loops to a minimum to avoid skewing timings

## Results Format

Benchmark results are saved in JSON format:
```json
{
  "timestamp": "2025-08-29T16:24:00Z",
  "zirc_version": "0.0.1-dev",
  "benchmarks": [
    {
      "name": "fibonacci",
      "iterations": 10,
      "avg_total_ms": 15.2,
      "min_total_ms": 14.8,
      "max_total_ms": 16.1,
      "avg_lex_ms": 0.4,
      "avg_parse_ms": 0.7,
      "avg_exec_ms": 14.1,
      "memory_usage_kb": 256
    }
  ]
}
```
