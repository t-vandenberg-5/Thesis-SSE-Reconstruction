use bumpalo::Bump;
use criterion::{Criterion, criterion_group, criterion_main};
use nanorand::WyRand;
use query_engine::buffer::{Buffer, BufferTrait};
use query_engine::generator::RandomGenerator;

fn criterion_benchmark(c: &mut Criterion) {
    let own = Bump::new();
    let mut buf = Buffer::<_, _, RandomGenerator<WyRand, 8>>::new(&own);

    c.bench_function("fresh", |b| {
        b.iter(|| {
            buf.next(&[100u8]);
        })
    });
    for _ in 0..100_000_000 {
        buf.next(&[100]);
    }
    buf.restart();
    c.bench_function("pp", |b| {
        b.iter(|| {
            buf.next(&[100]);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
