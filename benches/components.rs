use criterion::{criterion_group, criterion_main, Criterion};
use bumpalo::Bump;
use database_engine::DB;
use nanorand::WyRand;
use query_engine::{buffer::{Buffer, BufferTrait}, generator::RandomGenerator};

const DIMENSIONS: usize = 2;
type T = u16;


fn criterion_benchmark(c: &mut Criterion) {
    const DOMAINS: [T; DIMENSIONS] = [100; DIMENSIONS];
    let database = DB::generate(&DOMAINS);
    //c.bench_function("simd_gen_query", |b| b.iter(|| database.generate_query()));
    let bump = Bump::new();
    let mut qe: Buffer<'_, RandomGenerator<WyRand, 8>> = query_engine::buffer::Buffer::new(&bump);
    c.bench_function("query_gen_fresh", |b| b.iter(|| qe.next(&DOMAINS)));
    qe.restart();
    c.bench_function("simd_search", |b| b.iter(|| database.search(qe.next(&DOMAINS))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

