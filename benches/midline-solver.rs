use criterion::{Criterion, criterion_group, criterion_main};
use database_engine::DB;
use emptyblock::simd::simd_block_width;
use nanorand::WyRand;
use query_engine::generator::{GeneratorTrait, RandomGenerator};
use reconstructor::{Solver, midline::Tracker};

const DIMENSIONS: usize = 2;
type T = u16;

fn criterion_benchmark(c: &mut Criterion) {
    const DOMAINS: [T; DIMENSIONS] = [100, 80];
    let mut database = DB::generate(&DOMAINS);
    //c.bench_function("simd_gen_query", |b| b.iter(|| database.generate_query()));
    let mut qe: RandomGenerator<WyRand, 8> = RandomGenerator::default();
    let mut solver = Tracker::new(DOMAINS);
    c.bench_function("solve midline", |b| {
        b.iter(|| {
            while !solver.is_solved() {
                let query = qe.generate(&DOMAINS);
                if solver.add_query(&query, database.search(query), &database) {
                    panic!("Should not occur")
                }
            }
            database = DB::generate(&DOMAINS);
            solver = Tracker::new(DOMAINS);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
