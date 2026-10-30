use bumpalo::Bump;
use criterion::{Criterion, criterion_group, criterion_main};
use database_engine::DB;
use emptyblock::simd::simd_block_width;
use nanorand::WyRand;
use query_engine::{
    buffer::{Buffer, BufferTrait},
    generator::RandomGenerator,
};
use reconstructor::{Solver, midline::Tracker};

const DIMENSIONS: usize = 2;
type T = u16;

fn criterion_benchmark(c: &mut Criterion) {
    const DOMAINS: [T; DIMENSIONS] = [100; DIMENSIONS];
    let database = DB::generate(&DOMAINS);
    //c.bench_function("simd_gen_query", |b| b.iter(|| database.generate_query()));
    let bump = Bump::new();
    let mut qe: Buffer<'_, RandomGenerator<WyRand, 8>> = query_engine::buffer::Buffer::new(&bump);
    let mut solver = Tracker::new(DOMAINS);
    c.bench_function("solve", |b| {
        b.iter(|| {
            if !solver.is_solved() {
                let query = qe.next(&DOMAINS);
                if solver.add_query(&query, database.search(query), &database) {
                    qe.restart();
                }
            } else {
                solver = Tracker::new(DOMAINS);
                qe.restart();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
