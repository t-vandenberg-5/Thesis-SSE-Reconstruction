// use criterion::{criterion_group, criterion_main, Criterion};
// use nanorand::{BufferedRng, ChaCha8, Rng};
// use emptyblock::*;
// use consts::*;
// use std::array::from_fn;
//
//
// fn criterion_benchmark(c: &mut Criterion) {
//     let mut rand = BufferedRng::new(ChaCha8::new());
//     let pos: [T; DIMENSIONS] = from_fn(|_| rand.generate_range(0..DOMAIN));
//     let mut eb = naive::Naive::default();
//     c.bench_function("add_simple_empty", |b| b.iter(|| {
//         let query = from_fn(|_| {
//             let a = rand.generate_range(0..DOMAIN);
//             let b = rand.generate_range(0..DOMAIN);
//             if a < b {
//                 (a, b)
//             } else {
//                 (b, a)
//             }
//         });
//         eb.add_empty_area(query);
//     }));
//     eb = naive::Naive::default();
//     c.bench_function("add_empty", |b| b.iter(|| {
//         let query = from_fn(|_| {
//             let a = rand.generate_range(0..DOMAIN);
//             let b = rand.generate_range(0..DOMAIN);
//             if a < b {
//                 (a, b)
//             } else {
//                 (b, a)
//             }
//         });
//         let mut intersects = false;
//         for d in 0..DIMENSIONS {
//             if query[d].0 <= pos[d] && query[d].1 >= pos[d] {
//                 intersects = true;
//             }
//         }
//         if !intersects {
//             eb.add_empty_area(query);
//         }
//     }));
//     c.bench_function("find_pos", |b| b.iter(|| eb.is_single_pos()));
// }
//
// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
//
//
