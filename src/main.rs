use std::fs;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use cloud_vm::{CompressionChain, CompressedData, version};
use cloud_vm::ops::{RleOp, ZeroOp};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a file
    Compress {
        /// Input file to compress
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file for compressed data
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Decompress a file
    Decompress {
        /// Input file to decompress
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file for decompressed data
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // If no command is provided, just show version and exit
    if cli.command.is_none() {
        println!("Cloud-VM Compression v{}", version());
        return Ok(());
    }

    // Create compression chain with all available operations
    let mut chain = CompressionChain::new();
    chain.add_op(Box::new(ZeroOp));
    chain.add_op(Box::new(RleOp));

    match cli.command.unwrap() {
        Commands::Compress { input, output } => {
            // Read input file
            let data = fs::read(&input)?;
            
            // Compress data
            let compressed = chain.compress(&data)?;
            
            // Write compressed data
            let output_data = compressed.to_bytes();
            let compressed_size = output_data.len();
            fs::write(&output, &output_data)?;
            
            println!("Compressed {} -> {}", input.display(), output.display());
            println!("Original size: {} bytes", data.len());
            println!("Compressed size: {} bytes", compressed_size);
            println!("Version: {}", version());
            println!("Operations applied: {:?}", compressed.op_chain());
        },
        Commands::Decompress { input, output } => {
            // Read compressed file
            let input_data = fs::read(&input)?;
            
            // Parse compressed data
            let compressed = CompressedData::from_bytes(&input_data)?;
            
            if !compressed.is_compatible() {
                println!("Warning: File version {:?} may not be compatible with current version {}", 
                    compressed.version(), version());
            }
            
            // Decompress
            let decompressed = chain.decompress(compressed)?;
            
            // Write output
            fs::write(&output, decompressed)?;
            
            println!("Decompressed {} -> {}", input.display(), output.display());
            println!("Compressed size: {} bytes", input_data.len());
            println!("Decompressed size: {} bytes", fs::metadata(&output)?.len());
        },
    }

    Ok(())
}
