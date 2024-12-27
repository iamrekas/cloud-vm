# Cloud-VM EVM Compression

A Rust-based EVM compression system that provides modular, nestable compression operations with version tracking.

## Features

- **Modular Compression Operations**: Each compression operation is independent and can be chained together
- **Self-Evaluating**: Operations can determine if they can compress data further
- **Nestable**: Operations can be nested and chained in any order
- **Version Tracking**: Compressed data includes version information for future compatibility
- **Extensible**: Easy to add new compression operations by implementing the `CompressionOp` trait

## Installation

### Prerequisites

- Rust toolchain (1.70.0 or later)
- Cargo package manager

### Building from Source

```bash
# Clone the repository
git clone https://github.com/iamrekas/cloud-vm.git
cd cloud-vm

# Build the project
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Usage

### Command Line Interface

```bash
# Show version
cloud-vm --version

# Compress a file
cloud-vm compress -i input.file -o compressed.out

# Decompress a file
cloud-vm decompress -i compressed.out -o decompressed.file
```

### As a Library

```rust
use cloud_vm::{CompressionChain, CompressedData};
use cloud_vm::ops::{RleOp, ZeroOp};

// Create compression chain
let mut chain = CompressionChain::new();

// Add operations
chain.add_op(Box::new(ZeroOp));
chain.add_op(Box::new(RleOp));

// Compress data
let data = vec![1, 1, 1, 0, 0, 0, 0, 2, 2, 2];
let compressed = chain.compress(&data)?;

// Decompress
let decompressed = chain.decompress(compressed)?;
```

## Development

### Project Structure

```
cloud-vm/
├── src/
│   ├── lib.rs         # Core compression framework
│   ├── main.rs        # CLI implementation
│   └── ops/           # Compression operations
│       └── mod.rs     # Built-in operations
├── benches/           # Performance benchmarks
└── .github/           # GitHub Actions workflows
```

### Adding New Operations

1. Implement the `CompressionOp` trait:
```rust
struct CustomOp;

impl CompressionOp for CustomOp {
    fn op_code(&self) -> u8 {
        42 // Unique operation code
    }

    fn can_compress(&self, data: &[u8]) -> bool {
        // Implement compression detection logic
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // Implement compression logic
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // Implement decompression logic
    }
}
```

2. Add to compression chain:
```rust
let mut chain = CompressionChain::new();
chain.add_op(Box::new(CustomOp));
```

## Version Compatibility

The system maintains version information in compressed data to ensure future compatibility:

- Version 1: Initial implementation
  - Basic operation chaining
  - RLE and Zero compression operations

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see the [LICENSE](LICENSE) file for details
