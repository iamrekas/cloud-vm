use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cloud_vm::{CompressionChain, CompressedData};
use cloud_vm::ops::{RleOp, ZeroOp};

fn benchmark_compression(c: &mut Criterion) {
    let mut chain = CompressionChain::new();
    chain.add_op(Box::new(ZeroOp));
    chain.add_op(Box::new(RleOp));

    // Create test data with mix of patterns
    let mut data = Vec::with_capacity(1000);
    for i in 0..1000 {
        match i % 4 {
            0 => data.extend_from_slice(&[0, 0, 0, 0]),
            1 => data.extend_from_slice(&[1, 1, 1]),
            2 => data.extend_from_slice(&[2, 3, 4]),
            _ => data.extend_from_slice(&[5, 5, 5, 5, 5]),
        }
    }

    c.bench_function("compress mixed data", |b| {
        b.iter(|| {
            chain.compress(black_box(&data)).unwrap()
        })
    });

    let compressed = chain.compress(&data).unwrap();
    c.bench_function("decompress mixed data", |b| {
        b.iter(|| {
            chain.decompress(black_box(CompressedData::new(
                compressed.data().to_vec(),
                compressed.op_chain().to_vec()
            ))).unwrap()
        })
    });
}

criterion_group!(benches, benchmark_compression);
criterion_main!(benches);
