#![feature(portable_simd)]
use database_engine::{DB, qrsw};
use nanorand::{Rng, WyRand};
use reconstructor::{
    Solver, T, U,
    dataset::{CSVConfigBuilder, get_minimum, parse},
    midline::{self, Tracker},
};
use std::{
    simd::Simd,
    thread::{self, JoinHandle},
};

#[macro_export]
macro_rules! query {
    ( $x:expr ) => {{
        let mut d0: [u16; 2];
        let mut d1: [u16; 2];
        loop {
            let value: u64 = $x.generate();
            d0 = [(value & 0x7FFF) as u16, ((value >> 16) & 0x7FFF) as u16];
            d1 = [
                ((value >> 32) & 0x7FFF) as u16,
                ((value >> 48) & 0x7FFF) as u16,
            ];
            //println!("{:?}, {:?}", d0, d1);

            // Branchless version of `if d0[0] == dom[0] {d0[0] = d0[1]}`
            d0[0] = d0[(d0[0] == 25456) as usize];
            d1[0] = d1[(d1[0] == 24052) as usize];
            //println!("{:?}, {:?}", d0, d1);

            if d0[0] <= 25455 && d0[1] <= 25455 && d1[0] <= 24051 && d1[1] <= 24051 {
                break;
            }
        }
        //println!("passed: {:?}, {:?}", d0, d1);

        [
            (d0[0].min(d0[1]), d0[0].max(d0[1])),
            (d1[0].min(d1[1]), d1[0].max(d1[1])),
        ]
    }};
}

fn main() {
    let column1 = 4;
    let column2 = 5;
    let column3 = 6;
    let builder = CSVConfigBuilder::default()
        .filter(column1, |x| x.len() > 2)
        .filter(column1, |x| x[1..2] == *"." || x[2..3] == *".")
        .filter(column2, |x| x.len() > 2)
        .filter(column2, |x| x[1..2] == *"." || x[2..3] == *".")
        .filter(column3, |x| x.len() > 0)
        .filter(column3, |x| x != "null");
    let mut reader = builder.rb();
    let mut coll1 = Vec::new();
    let mut coll2 = Vec::new();
    let mut coll3 = Vec::new();
    for s in builder.read_iter(&mut reader) {
        coll1.push(s.get(column1).unwrap().to_string());
        coll2.push(s.get(column2).unwrap().to_string());
        coll3.push(s.get(column3).unwrap().to_string());
    }
    println!("{}", coll1.len());
    println!("{}", coll2.len());
    println!("{}", coll3.len());
    let m1 = get_minimum(&mut coll1.clone());
    let m2 = get_minimum(&mut coll2.clone());
    let mut c1 = Vec::with_capacity(coll1.len());
    let mut c2 = Vec::with_capacity(coll2.len());
    let mut c3 = Vec::with_capacity(coll3.len());
    for i in 0..coll1.len() {
        c1.push(parse(&coll1[i], m1));
        c2.push(parse(&coll2[i], m2));
        c3.push(T::try_from(coll3[i].parse::<u16>().unwrap() / 30).unwrap());
    }
    println!("{}", c1.len());

    const DIMENSIONS: usize = 2;
    const CAPACITY: usize = 256;
    const QRMW: usize = qrsw(CAPACITY);
    let dummy_simd = Simd::from_array([0usize; QRMW]);
    const THREADS: usize = 25;
    const TESTS_PER_THREAD: usize = 4;
    const DOMAINS: [T; DIMENSIONS] = [25456, 24052];
    let handle: [JoinHandle<[usize; TESTS_PER_THREAD]>; THREADS] = core::array::from_fn(
        |_threadnum| {
            let v1 = c1.clone();
            let v2 = c2.clone();
            thread::spawn(move || {
                let zipped: [usize; TESTS_PER_THREAD] = core::array::from_fn(|_| {
                    let sqe = WyRand::new();
                    let mut max_queries = 0;
                    let mut bound_start = 0;
                    while bound_start + 256 <= v1.len() {
                        let database = DB::<T, U, DIMENSIONS, CAPACITY>::read(&[
                            v1[bound_start..bound_start + CAPACITY].to_vec(),
                            v2[bound_start..bound_start + CAPACITY].to_vec(),
                            //v3[bound_start..bound_start + CAPACITY].to_vec()
                        ]);
                        //println!("[{:?}] Thread {} read database {}..{}", chrono::Utc::now(), _threadnum, bound_start, bound_start + CAPACITY);
                        let mut qe = sqe.clone();
                        let mut solver: midline::Tracker<DIMENSIONS, QRMW> =
                            midline::Tracker::new::<CAPACITY>(DOMAINS);
                        let mut queries = 0;
                        while !<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver) {
                        queries += 1;
                        let query = query!(qe);
                        solver.add_query(&query, dummy_simd, &database);
                        if queries % 100_000_000 == 0 {
                            // if solver.remaining() < 5 {
                            //     println!("[{:?}] Thread {} finished {} queries, {} records remaining: {:?}", chrono::Utc::now(), _threadnum, queries, solver.remaining(), solver.remaining_pos());
                            //     for d in solver.remaining_pos() {
                            //         for r in d {
                            //             println!("Thread {} {} queries record pos: {}, {}", _threadnum, queries, v1[bound_start + r], v2[bound_start + r]);
                            //         }
                            //     }
                            // } else {
                            //     println!("[{:?}] Thread {} finished {} queries, {} records remaining", chrono::Utc::now(), _threadnum, queries, solver.remaining());
                            // }
                        }
                    }

                        for i in 0..CAPACITY {
                            assert_eq!(<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::get_solved(&solver, i as U), Some(database.cheat_pos_traverse(i as U)));
                        }

                        if queries > max_queries {
                            max_queries = queries;
                        }
                        bound_start += 256;
                    }

                    let database = DB::<T, U, DIMENSIONS, 68>::read(&[
                        v1[bound_start..bound_start + 68].to_vec(),
                        v2[bound_start..bound_start + 68].to_vec(),
                        //v3[bound_start..bound_start + 68].to_vec()
                    ]);
                    //println!("Read Database");
                    let mut qe = sqe.clone();
                    let mut solver: midline::Tracker<DIMENSIONS, QRMW> =
                        midline::Tracker::new::<68>(DOMAINS);
                    let mut queries = 0;
                    while !<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, 68, QRMW>>::is_solved(
                        &solver,
                    ) {
                        queries += 1;
                        let query = query!(qe);
                        solver.add_query(&query, database.search(query), &database);
                    }

                    for i in 0..68 {
                        assert_eq!(
                            <Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, 68, QRMW>>::get_solved(
                                &solver, i as U
                            ),
                            Some(database.cheat_pos_traverse(i as U))
                        );
                    }

                    if queries > max_queries {
                        max_queries = queries;
                    }

                    println!("max_queries: {max_queries}");
                    max_queries
                });
                zipped
            })
        },
    );
    let mut r: Vec<usize> = Vec::with_capacity(TESTS_PER_THREAD * THREADS);

    for h in handle {
        let data = h.join().unwrap();
        r.extend_from_slice(&data);
    }

    r.sort();
    match TESTS_PER_THREAD * THREADS {
        100 => {
            println!("p50: {:?}, p90: {:?}, p99: {:?}", r[50], r[90], r[99]);
        }
        _ => {
            println!("full: {:?}", r);
        }
    }
}
