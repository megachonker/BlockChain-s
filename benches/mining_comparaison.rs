use block_chain::{mine, mine_hasher_clone,Block, mine_hasher_lessrng}; // Import the functions from the block_chain module

use criterion::{criterion_group, criterion_main, Criterion};

// Define the benchmarking function
fn benchmark_functions(c: &mut Criterion) {
    let fist_block = Block::new(vec![]);
    c.bench_function("Function Mine restart hasher", |b| b.iter(|| mine(&fist_block)));
    c.bench_function("Function Mine clone hasher", |b| b.iter(|| mine_hasher_clone(&fist_block)));
    c.bench_function("Function Mine clone less rng", |b| b.iter(|| mine_hasher_lessrng(&fist_block)));
}

// Specify the criterion group and main function
criterion_group!(benches, benchmark_functions);
criterion_main!(benches);

