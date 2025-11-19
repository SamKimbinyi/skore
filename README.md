# skore

[![Rust](https://github.com/SamKimbinyi/skore/actions/workflows/rust.yml/badge.svg)](https://github.com/SamKimbinyi/skore/actions/workflows/rust.yml)

A key-value store implementation in Rust. This is a learning project exploring storage engines, memory-mapped files, and
database fundamentals.

## What's This?

Skore is a simple key-value database that lets you store and retrieve byte arrays. ThinkocksDB, but way simpler and
built to understand how these systems work under the hood.

The project uses memory-mapped files with rkyv for zero-copy serialization, making it reasonably fast for a learning
project. There's also an in-memory store for testing and quick prototyping.

## Project Structure

```
crates/
├── skore-core/      # Core traits and error handling
├── skore-storage/   # Storage implementations (memory, file-based)
├── skore-cli/       # Command-line interface (WIP)
└── skore/           # Main library with public API
```

## Current Features

- In-memory storage with BTreeMap backing
- File-based storage with memory-mapped I/O
- Append-only log structure
- Tombstone-based deletions
- Index rebuilding on startup

## Usage

```rust
use skore::Skore;

let mut store = Skore::default ();

// Basic operations
store.set(b"user:123", b"alice") ?;
let value = store.get(b"user:123") ?; // Some(b"alice")
store.delete(b"user:123") ?;
```

## What's Next

- Compaction to reclaim space from deleted entries
- Write-ahead log for durability guarantees
- Range queries and iterators
- Bloom filters for faster lookups
- Benchmarking
- CLI tool for interactive use
- IO over network

## Running Tests

```bash
cargo test
```

