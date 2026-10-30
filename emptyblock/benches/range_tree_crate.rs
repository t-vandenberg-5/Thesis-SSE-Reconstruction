use btree_range_map::RangeMap;
use criterion::{Criterion, criterion_group, criterion_main};
use nanorand::{BufferedRng, ChaCha8, Rng};

fn criterion_benchmark(c: &mut Criterion) {
    let mut rand = BufferedRng::new(ChaCha8::new());
    c.bench_function("add_empty", |b| {
        b.iter(|| {
            let mut rm = RangeMap::new();
            for _d in 0..10 {
                let query = {
                    let a = rand.generate_range(0..4096);
                    let b = rand.generate_range(0..4096);
                    if a < b { a..=b } else { b..=a }
                };
                rm.insert(query, true);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
