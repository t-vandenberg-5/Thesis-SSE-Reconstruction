use criterion::{criterion_group, criterion_main, Criterion};
use database_engine::{DB, qrsw};
use nanorand::{ChaCha8, WyRand};
use query_engine::generator::{GeneratorTrait, RandomGenerator};


fn criterion_benchmark(c: &mut Criterion) {
    const DOMAINS: [u8; 1] = [100];
    let database = DB::<_, u16, _, 100>::generate(&DOMAINS);
    //c.bench_function("simd_gen_query", |b| b.iter(|| database.generate_query()));
    let mut qe = RandomGenerator::<WyRand, 8>::default();
    c.bench_function("simd_search", |b| b.iter(|| database.search::<{ qrsw(100) }>(qe.generate(&DOMAINS))));
    c.bench_function("simd_qg", |b| b.iter(|| qe.generate(&DOMAINS)));
    let mut qe = RandomGenerator::<ChaCha8, 64>::default();
    c.bench_function("simd_search_chacha", |b| b.iter(|| database.search::<{ qrsw(100) }>(qe.generate(&DOMAINS))));
    c.bench_function("simd_qg_chacha", |b| b.iter(|| qe.generate(&DOMAINS)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

