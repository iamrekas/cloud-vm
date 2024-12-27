//! # Cloud-VM EVM Compression
//! 
//! A modular compression system designed for EVM data with support for:
//! - Nestable compression operations
//! - Version tracking
//! - Operation chaining
//! - Self-evaluating compression capabilities
//! 
//! ## Basic Usage
//! 
//! ```rust
//! use cloud_vm::{CompressionChain, CompressedData};
//! use cloud_vm::ops::{RleOp, ZeroOp};
//! 
//! // Create chain with multiple operations
//! let mut chain = CompressionChain::new();
//! chain.add_op(Box::new(ZeroOp));
//! chain.add_op(Box::new(RleOp));
//! 
//! // Compress data
//! let data = vec![1, 1, 1, 0, 0, 0, 0, 2, 2, 2];
//! let compressed = chain.compress(&data).unwrap();
//! 
//! // Decompress
//! let decompressed = chain.decompress(compressed).unwrap();
//! assert_eq!(data, decompressed);
//! ```

use std::error::Error;

pub mod ops;

use semver::Version;

/// Get the current version of the compression system
pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
}

/// Version information stored in compressed files
#[derive(Debug, Clone, Copy)]
#[derive(PartialEq)]
pub struct FileVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl PartialEq<u8> for FileVersion {
    fn eq(&self, other: &u8) -> bool {
        self.major == *other
    }
}

impl FileVersion {
    /// Create a new FileVersion from the current crate version
    pub fn current() -> Self {
        let v = version();
        Self {
            major: v.major as u8,
            minor: v.minor as u8,
            patch: v.patch as u8,
        }
    }

    /// Convert version to bytes for storage
    pub fn to_bytes(&self) -> [u8; 3] {
        [self.major, self.minor, self.patch]
    }

    /// Create version from bytes
    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        Self {
            major: bytes[0],
            minor: bytes[1],
            patch: bytes[2],
        }
    }

    /// Check if this version is compatible with current version
    pub fn is_compatible(&self) -> bool {
        let current = Self::current();
        self.major == current.major
    }
}

/// Current version number used in compressed files (major version only)
pub const CURRENT_VERSION: u8 = {
    let version = env!("CARGO_PKG_VERSION");
    let mut result = 0u8;
    let mut i = 0;
    while i < version.len() && version.as_bytes()[i] >= b'0' && version.as_bytes()[i] <= b'9' {
        result = result * 10 + (version.as_bytes()[i] - b'0');
        i += 1;
    }
    result
};

/// Represents compressed data with version and operation tracking
#[derive(Debug)]
pub struct CompressedData {
    /// Version of the compression system used
    version: FileVersion,
    /// Chain of operation codes used for compression
    op_chain: Vec<u8>,
    /// The compressed data
    data: Vec<u8>,
}

impl CompressedData {
    /// Creates new compressed data with current version
    pub fn new(data: Vec<u8>, op_chain: Vec<u8>) -> Self {
        Self {
            version: FileVersion::current(),
            op_chain,
            data,
        }
    }

    /// Returns the version of the compression
    pub fn version(&self) -> FileVersion {
        self.version
    }

    /// Returns a reference to the compressed data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the chain of operation codes used
    pub fn op_chain(&self) -> &[u8] {
        &self.op_chain
    }

    /// Create CompressedData from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        if bytes.len() < 4 { // 3 bytes version + 1 byte chain length
            return Err("Invalid compressed data format".into());
        }

        let version = FileVersion::from_bytes([bytes[0], bytes[1], bytes[2]]);
        let chain_len = bytes[3] as usize;

        if bytes.len() < 4 + chain_len {
            return Err("Invalid compressed data format".into());
        }

        let op_chain = bytes[4..4+chain_len].to_vec();
        let data = bytes[4+chain_len..].to_vec();

        Ok(Self {
            version,
            op_chain,
            data,
        })
    }

    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.version.to_bytes());
        result.push(self.op_chain.len() as u8);
        result.extend_from_slice(&self.op_chain);
        result.extend_from_slice(&self.data);
        result
    }

    /// Check if this compressed data is compatible with current version
    pub fn is_compatible(&self) -> bool {
        self.version.is_compatible()
    }
}

/// Trait for implementing compression operations
pub trait CompressionOp {
    /// Returns unique operation code
    fn op_code(&self) -> u8;
    
    /// Determines if this operation can compress the data further
    fn can_compress(&self, data: &[u8]) -> bool;
    
    /// Compresses the data
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    
    /// Decompresses the data
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
}

/// Custom error type for compression operations
#[derive(Debug)]
pub struct CompressionError(String);

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compression error: {}", self.0)
    }
}

impl Error for CompressionError {}

/// Manages chain of compression operations
pub struct CompressionChain {
    ops: Vec<Box<dyn CompressionOp>>,
}

impl CompressionChain {
    /// Creates new empty compression chain
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    /// Adds compression operation to the chain
    pub fn add_op(&mut self, op: Box<dyn CompressionOp>) {
        self.ops.push(op);
    }

    /// Compresses data using all applicable operations in chain
    pub fn compress(&self, data: &[u8]) -> Result<CompressedData, Box<dyn Error>> {
        let mut current_data = data.to_vec();
        let mut op_chain = Vec::new();

        for op in &self.ops {
            if op.can_compress(&current_data) {
                current_data = op.compress(&current_data)?;
                op_chain.push(op.op_code());
            }
        }

        Ok(CompressedData::new(current_data, op_chain))
    }

    /// Decompresses data using stored operation chain
    pub fn decompress(&self, compressed: CompressedData) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut current_data = compressed.data().to_vec();
        
        // Decompress in reverse order
        for &op_code in compressed.op_chain().iter().rev() {
            let op = self.ops.iter()
                .find(|op| op.op_code() == op_code)
                .ok_or_else(|| CompressionError("Unknown operation code".into()))?;
                
            current_data = op.decompress(&current_data)?;
        }

        Ok(current_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock compression op for testing
    struct MockOp;

    impl CompressionOp for MockOp {
        fn op_code(&self) -> u8 {
            1
        }

        fn can_compress(&self, data: &[u8]) -> bool {
            !data.is_empty()
        }

        fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
            // Simple mock compression: duplicate each byte
            Ok(data.iter().flat_map(|&b| vec![b, b]).collect())
        }

        fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
            // Take every other byte
            Ok(data.iter().step_by(2).copied().collect())
        }
    }

    #[test]
    fn test_compression_chain() {
        let mut chain = CompressionChain::new();
        chain.add_op(Box::new(MockOp));

        let original_data = vec![1, 2, 3];
        let compressed = chain.compress(&original_data).unwrap();
        
        assert_eq!(compressed.version(), CURRENT_VERSION);
        assert_eq!(compressed.op_chain(), &[1]);
        
        let decompressed = chain.decompress(compressed).unwrap();
        assert_eq!(decompressed, original_data);
    }
}
