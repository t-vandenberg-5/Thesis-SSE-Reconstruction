use criterion::{criterion_group, criterion_main, Criterion};
use nanorand::{BufferedRng, ChaCha8, Rng};
use emptyblock::{simd::simd_block_width, *};
use std::array::from_fn;


fn criterion_benchmark(c: &mut Criterion) {
    let mut rand = BufferedRng::new(ChaCha8::new());
    const SBW: usize = simd_block_width(4096).0;
    const SLL: usize = simd_block_width(4096).1;
    let mut a = simd::SimdBlock2D::<u16, 4096, SBW, SLL>::default();
    c.bench_function("add_thingy", |b| b.iter(|| {
        let query = from_fn(|_| {
            let a = rand.generate_range(0..4096);
            let b = rand.generate_range(0..4096);
            if a < b {
                (a, b)
            } else {
                (b, a)
            }
        });
        a.add_area(&query);
    }));
    c.bench_function("get_pos", |b| b.iter(|| {
        let query = from_fn(|_| {
            let a = rand.generate_range(0..4096);
            let b = rand.generate_range(0..4096);
            if a < b {
                (a, b)
            } else {
                (b, a)
            }
        });
        a.get_pos(&query);
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);


