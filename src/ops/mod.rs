//! Compression operations implementations
//! 
//! This module contains the built-in compression operations:
//! - RLE (Run-Length Encoding) for repeated byte sequences
//! - Zero compression for sequences of zeros

use crate::{CompressionOp, Error};

/// Run-Length Encoding compression operation
pub struct RleOp;

impl CompressionOp for RleOp {
    fn op_code(&self) -> u8 {
        1
    }

    fn can_compress(&self, data: &[u8]) -> bool {
        if data.len() < 3 {
            return false;
        }

        // Check if there are any repeated sequences
        data.windows(3).any(|w| w[0] == w[1] && w[1] == w[2])
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let current = data[i];
            let mut count = 1;

            // Count repeated bytes
            while i + count < data.len() && data[i + count] == current && count < 255 {
                count += 1;
            }

            if count >= 3 {
                // Store as count + value
                compressed.push(count as u8);
                compressed.push(current);
                i += count;
            } else {
                // Store as literal
                compressed.push(1);
                compressed.push(current);
                i += 1;
            }
        }

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let count = data[i] as usize;
            let value = data[i + 1];

            for _ in 0..count {
                decompressed.push(value);
            }

            i += 2;
        }

        Ok(decompressed)
    }
}

/// Zero sequence compression operation
pub struct ZeroOp;

impl CompressionOp for ZeroOp {
    fn op_code(&self) -> u8 {
        2
    }

    fn can_compress(&self, data: &[u8]) -> bool {
        data.windows(4).any(|w| w.iter().all(|&b| b == 0))
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if i + 3 < data.len() && data[i..i+4].iter().all(|&b| b == 0) {
                let mut count = 4;
                while i + count < data.len() && data[i + count] == 0 && count < 255 {
                    count += 1;
                }
                compressed.push(0); // Zero marker
                compressed.push(count as u8);
                i += count;
            } else {
                compressed.push(data[i]);
                i += 1;
            }
        }

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 0 {
                let count = data[i + 1] as usize;
                decompressed.extend(std::iter::repeat(0).take(count));
                i += 2;
            } else {
                decompressed.push(data[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_compression() {
        let op = RleOp;
        let data = vec![1, 1, 1, 2, 3, 3, 3, 3, 4];
        
        assert!(op.can_compress(&data));
        
        let compressed = op.compress(&data).unwrap();
        let decompressed = op.decompress(&compressed).unwrap();
        
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_zero_compression() {
        let op = ZeroOp;
        let data = vec![1, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 3];
        
        assert!(op.can_compress(&data));
        
        let compressed = op.compress(&data).unwrap();
        let decompressed = op.decompress(&compressed).unwrap();
        
        assert_eq!(data, decompressed);
    }
}
