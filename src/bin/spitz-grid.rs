use bumpalo::Bump;
use database_engine::{DB, qrsw};
use reconstructor::{Solver, T, U, Tracker2D};
use emptyblock::simd::simd_block_width;
use nanorand::WyRand;
use query_engine::{buffer::{Buffer, BufferTrait}, generator::RandomGenerator};

fn main() {
    const DIMENSIONS: usize = 2;
    const DOMAINS: [T; DIMENSIONS] = [34133, 27364];
    const CAPACITY: usize = 512;
    const QRMW: usize = qrsw(CAPACITY);
    const SBW: usize = simd_block_width(DOMAINS[1] as usize).0;
    const SLL: usize = simd_block_width(DOMAINS[1] as usize).1;
    let database = DB::<T, U, DIMENSIONS, CAPACITY>::generate(&DOMAINS);
    let bump = Bump::new();
    let mut qe = Buffer::<T, DIMENSIONS, RandomGenerator<WyRand, 8>>::new(&bump);
    let mut solver: Tracker2D<{ DOMAINS[0] as usize }, QRMW, { simd_block_width(DOMAINS[1] as usize).0 }, { simd_block_width(DOMAINS[1] as usize).1 } > = Tracker2D::new::<CAPACITY>(DOMAINS);
    while !<Tracker2D<{ DOMAINS[0] as usize }, QRMW, SBW, SLL> as Solver<2, CAPACITY, QRMW>>::is_solved(&solver) {
        let query = qe.next(&DOMAINS);
        if solver.add_query(&query, database.search(query), &database) {
            qe.restart();
            solver.reset_current_area(DOMAINS);
        }
    }

    for i in 0..CAPACITY {
        assert_eq!(<Tracker2D<{ DOMAINS[0] as usize }, QRMW, SBW, SLL> as Solver<2, CAPACITY, QRMW>>::get_solved(&solver, i as U), Some(database.cheat_pos_traverse(i as U)));
    }

    println!("{}", qe.size());
}
