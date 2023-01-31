use criterion::{criterion_group, criterion_main};

mod parse_bench;

criterion_group!(benches, parse_bench::bench_parser,);
criterion_main!(benches);
