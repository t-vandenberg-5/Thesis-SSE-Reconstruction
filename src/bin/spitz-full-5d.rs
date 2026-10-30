use reconstructor::{T, dataset::{CSVConfigBuilder, get_minimum, parse}, midline_fast};
use std::thread::{self, JoinHandle};
use nanorand::{Rng, WyRand};

#[macro_export]
macro_rules! query {
    ( $x:expr ) => {
        {
            let value_preload: u64 = $x.generate();
            let mut value: u64 = $x.generate();
            let mut value_index = 0;
            let bounds = [25456, 25455, 24052, 24051, 12, 11, 3, 2, 9, 8];
            let widths = [15, 15, 15, 15, 4, 4, 2, 2, 4, 4];
            let masks = [0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF, 0xF, 0xF, 0x3, 0x3, 0xF, 0xF];
            let mut index = 0;
            let mut raw = [0u16; 10];

            'preload: while index < 10 {
                raw[index] = ((value_preload >> value_index) & masks[index]) as T;
                if value_index + widths[index] >= 64 {
                    raw[index] |= ((value << (64 - value_index)) & masks[index]) as T;
                    value_index = (value_index + widths[index]) % 64;
                    index += (raw[index] <= bounds[index]) as usize;
                    break 'preload;
                }
                value_index = (value_index + widths[index]) % 64;
                index += (raw[index] <= bounds[index]) as usize;
            }

            while index < 10 {
                raw[index] = ((value_preload >> value_index) & masks[index]) as T;
                if value_index + widths[index] >= 64 {
                    value = $x.generate();
                    raw[index] |= ((value << (64 - value_index)) & masks[index]) as T;
                }
                value_index = (value_index + widths[index]) % 64;
                index += (raw[index] <= bounds[index]) as usize;
            }

            raw[0] = raw[(raw[0] == 25456) as usize];
            raw[2] = raw[(raw[2] == 24052) as usize];
            raw[4] = raw[(raw[4] == 12) as usize];
            raw[6] = raw[(raw[6] == 3) as usize];
            raw[8] = raw[(raw[8] == 9) as usize];

            [
                (raw[0].min(raw[1]), raw[0].max(raw[1])),
                (raw[2].min(raw[3]), raw[2].max(raw[3])),
                (raw[4].min(raw[5]), raw[4].max(raw[5])),
                (raw[6].min(raw[7]), raw[6].max(raw[7])),
                (raw[8].min(raw[9]), raw[8].max(raw[9])),
            ]
        }
    };
}

fn main() {
    let column1 = 4;
    let column2 = 5;
    let column3 = 6;
    let column4 = 3;
    let column5 = 2;
    let builder = CSVConfigBuilder::default()
        .filter(column1, |x| x.len() > 2)
        .filter(column1, |x| x[1..2] == *"." || x[2..3] == *".")
        .filter(column2, |x| x.len() > 2)
        .filter(column2, |x| x[1..2] == *"." || x[2..3] == *".")
        .filter(column3, |x| x.len() > 0)
        .filter(column3, |x| x != "null")
        ;
    let mut reader = builder.rb();
    let mut coll1 = Vec::new();
    let mut coll2 = Vec::new();
    let mut coll3 = Vec::new();
    let mut coll4 = Vec::new();
    let mut coll5 = Vec::new();
    for s in builder.read_iter(&mut reader) {
        coll1.push(s.get(column1).unwrap().to_string());
        coll2.push(s.get(column2).unwrap().to_string());
        coll3.push(s.get(column3).unwrap().to_string());
        coll4.push(s.get(column4).unwrap().to_string());
        coll5.push(s.get(column5).unwrap().to_string());
    }
    println!("{}", coll1.len());
    println!("{}", coll2.len());
    println!("{}", coll3.len());
    println!("{}", coll4.len());
    println!("{}", coll5.len());
    let m1 = get_minimum(&mut coll1.clone());
    let m2 = get_minimum(&mut coll2.clone());
    let mut c1 = Vec::with_capacity(coll1.len());
    let mut c2 = Vec::with_capacity(coll2.len());
    let mut c3 = Vec::with_capacity(coll3.len());
    let mut c4 = Vec::with_capacity(coll4.len());
    let mut c5 = Vec::with_capacity(coll5.len());
    for i in 0..coll1.len() {
        c1.push(parse(&coll1[i], m1));
        c2.push(parse(&coll2[i], m2));
        c3.push(T::try_from(coll3[i].parse::<u16>().unwrap() / 30).unwrap());
        c4.push(match coll4[i].as_str() {
            "eingehend" => 1,
            "ausgehend" => 2,
            "" => 0,
            _ => panic!("unknown value"),
        });
        c5.push(match coll5[i].as_str() {
            "GPRS" => 0,
            "SMS" => 1,
            "Telefonie" => 2,
            "Telefonie, CCBS" => 3,
            "Telefonie, CFB" => 4,
            "Telefonie, CFNRc" => 5,
            "Telefonie, CFNRy" => 6,
            "Telefonie, CH" => 7,
            "Telefonie, CLIP" => 8,
            _ => panic!("unknown value"),
        });
    }
    println!("{}", c1.len());


    const DIMENSIONS: usize = 5;
    const DIMENSIONS_MINUS_ONE: usize = DIMENSIONS - 1;
    const THREADS: usize = 25;
    const TESTS_PER_THREAD: usize = 4;
    let handle: [JoinHandle<[usize; TESTS_PER_THREAD]>; THREADS] = core::array::from_fn(|_threadnum| {
        let v1 = c1.clone();
        let v2 = c2.clone();
        let v3 = c3.clone();
        let v4 = c4.clone();
        let v5 = c5.clone();
        thread::spawn(move || {
            let zipped: [usize; TESTS_PER_THREAD] = core::array::from_fn(|_| {
                let mut sqe = WyRand::new();
                let mut queries = 0;
                let mut solver = midline_fast::Solver::<29252, DIMENSIONS>::new::<DIMENSIONS_MINUS_ONE>(&[v1.clone(), v2.clone(), v3.clone(), v4.clone(), v5.clone()]);
                // println!("Loaded");
                // println!("{}, {}", queries, solver.remaining());
                while !solver.is_solved() {
                    queries += 1;
                    let query = query!(sqe);
                    solver.process::<DIMENSIONS_MINUS_ONE>(query, 0);
                    solver.process::<DIMENSIONS_MINUS_ONE>(query, 1);
                    solver.process::<DIMENSIONS_MINUS_ONE>(query, 2);
                    solver.process::<DIMENSIONS_MINUS_ONE>(query, 3);
                    solver.process::<DIMENSIONS_MINUS_ONE>(query, 4);
                    // if queries % 1000_000 == 0 {
                    //     println!("{}, {}", queries, solver.remaining());
                    // }
                }
                println!("{queries}");

                queries
                //
                // let mut bound_start = 0;
                // while bound_start + 256 <= v1.len() {
                //     let database = DB::<T, U, DIMENSIONS, CAPACITY>::read(&[
                //         v1[bound_start..bound_start + CAPACITY].to_vec(),
                //         v2[bound_start..bound_start + CAPACITY].to_vec(),
                //         //v3[bound_start..bound_start + CAPACITY].to_vec()
                //     ]);
                //     //println!("[{:?}] Thread {} read database {}..{}", chrono::Utc::now(), _threadnum, bound_start, bound_start + CAPACITY);
                //     let mut qe = sqe.clone();
                //     let mut solver: midline::Tracker<DIMENSIONS, QRMW> = midline::Tracker::new::<CAPACITY>(DOMAINS);
                //     let mut queries = 0;
                //     while !<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver) {
                //         queries += 1;
                //         let query = query!(qe);
                //         solver.add_query(&query, dummy_simd, &database);
                //         if queries % 100_000_000 == 0 {
                //             // if solver.remaining() < 5 {
                //             //     println!("[{:?}] Thread {} finished {} queries, {} records remaining: {:?}", chrono::Utc::now(), _threadnum, queries, solver.remaining(), solver.remaining_pos());
                //             //     for d in solver.remaining_pos() {
                //             //         for r in d {
                //             //             println!("Thread {} {} queries record pos: {}, {}", _threadnum, queries, v1[bound_start + r], v2[bound_start + r]);
                //             //         }
                //             //     }
                //             // } else {
                //             //     println!("[{:?}] Thread {} finished {} queries, {} records remaining", chrono::Utc::now(), _threadnum, queries, solver.remaining());
                //             // }
                //         }
                //     }
                //
                //     for i in 0..CAPACITY {
                //         assert_eq!(<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::get_solved(&solver, i as U), Some(database.cheat_pos_traverse(i as U)));
                //     }
                //
                //     if queries > max_queries {
                //         max_queries = queries;
                //     }
                //     bound_start += 256;
                // }
                //
                // let database = DB::<T, U, DIMENSIONS, 68>::read(&[
                //     v1[bound_start..bound_start + 68].to_vec(),
                //     v2[bound_start..bound_start + 68].to_vec(),
                //     //v3[bound_start..bound_start + 68].to_vec()
                // ]);
                // //println!("Read Database");
                // let mut qe = sqe.clone();
                // let mut solver: midline::Tracker<DIMENSIONS, QRMW> = midline::Tracker::new::<68>(DOMAINS);
                // let mut queries = 0;
                // while !<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, 68, QRMW>>::is_solved(&solver) {
                //     queries += 1;
                //     let query = query!(qe);
                //     solver.add_query(&query, database.search(query), &database);
                // }
                //
                // for i in 0..68 {
                //     assert_eq!(<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, 68, QRMW>>::get_solved(&solver, i as U), Some(database.cheat_pos_traverse(i as U)));
                // }
                //
                // if queries > max_queries {
                //     max_queries = queries;
                // }
                //
                // println!("max_queries: {max_queries}");
                // max_queries
            });
            zipped
    })});
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
