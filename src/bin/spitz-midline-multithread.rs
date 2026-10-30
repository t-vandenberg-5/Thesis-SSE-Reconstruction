#![feature(portable_simd)]
#![feature(mpmc_channel)]
use database_engine::{DB, qrsw};
use nanorand::{Rng, WyRand};
use reconstructor::{
    Solver, T, U,
    dataset::{CSVConfigBuilder, get_minimum, parse},
    midline::{self, Tracker},
    midline_optimized::{self, Slate, hyperoctants},
};
use std::{
    simd::Simd,
    sync::mpmc::{self, Receiver, Sender},
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
            d0[0] = d0[(d0[0] == 25457) as usize];
            d1[0] = d1[(d1[0] == 24053) as usize];
            //println!("{:?}, {:?}", d0, d1);

            if d0[0] < 25457 && d0[1] < 25457 && d1[0] < 24053 && d1[1] < 24053 {
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

fn load_db_columns() -> (Vec<u16>, Vec<u16>) {
    let column1 = 4;
    let column2 = 5;
    let builder = CSVConfigBuilder::default()
        .filter(column1, |x| x.len() > 2)
        .filter(column1, |x| x[1..2] == *"." || x[2..3] == *".")
        .filter(column2, |x| x.len() > 2)
        .filter(column2, |x| x[1..2] == *"." || x[2..3] == *".");
    let mut reader = builder.rb();
    let mut coll1 = Vec::new();
    let mut coll2 = Vec::new();
    for s in builder.read_iter(&mut reader) {
        coll1.push(s.get(column1).unwrap().to_string());
        coll2.push(s.get(column2).unwrap().to_string());
    }
    println!("Load database: {}", coll1.len());
    println!("Load database: {}", coll2.len());
    let m1 = get_minimum(&mut coll1.clone()) - 1;
    let m2 = get_minimum(&mut coll2.clone()) - 1;
    let mut c1 = Vec::with_capacity(coll1.len());
    let mut c2 = Vec::with_capacity(coll2.len());
    for i in 0..coll1.len() {
        c1.push(parse(&coll1[i], m1));
        c2.push(parse(&coll2[i], m2));
    }
    println!("Load database: {}", c1.len());
    (c1, c2)
}

fn load_database_2d(
    c: (Vec<T>, Vec<T>),
) -> (
    midline_optimized::Slate<{ 2 - 1 }, 2, { hyperoctants(2) }>,
    [T; 2],
) {
    const DIMENSIONS: usize = 2;
    const CAPACITY: usize = 30363;
    const DOMAINS: [T; DIMENSIONS] = [25457, 24053];
    let mut solver = midline_optimized::Slate::<
        { DIMENSIONS - 1 },
        DIMENSIONS,
        { midline_optimized::hyperoctants(DIMENSIONS) },
    >::with_capacity(CAPACITY);
    let mut midpoints: [T; 2] = core::array::from_fn(|x| DOMAINS[x] / 2);
    for i in 0..CAPACITY {
        solver.insert(&mut [c.0[i], c.1[i]], &mut midpoints);
    }
    (solver, midpoints)
}

fn spawn_worker_2d(
    tx: Sender<[(u16, u16); 2]>,
    rx: Receiver<[(u16, u16); 2]>,
    s: Slate<{ 2 - 1 }, 2, { hyperoctants(2) }>,
    midpoints: [T; 2],
) -> JoinHandle<u64> {
    let thread_tx = tx.clone();
    let thread_rx = rx.clone();
    let mut thread_s = s.clone();
    let mut thread_midpoints = midpoints.clone();

    let child = thread::spawn(move || {
        let mut rand = WyRand::new();
        let mut queries = 0;
        while !thread_s.is_solved() {
            let mut query = query!(rand);
            queries += 1;
            if thread_s.process_query(&mut query, &mut thread_midpoints) {
                thread_tx.send(query).unwrap();
                while let Ok(mut query) = thread_rx.try_recv() {
                    thread_s.process_query(&mut query, &mut thread_midpoints);
                }
            }

            if queries % 1_000_000_000 == 0 {
                while let Ok(mut query) = thread_rx.try_recv() {
                    thread_s.process_query(&mut query, &mut thread_midpoints);
                }
                if queries % 100_000_000_000 == 0 {
                    println!("Current solver state: {:?}", thread_s);
                }
                println!(
                    "A thread finished {} queries, remaining len: {}",
                    queries,
                    thread_s.len()
                );
            }
        }

        queries
    });

    child
}

fn load_database_2d_reconstructor(c: (Vec<T>, Vec<T>), bound_start: usize) -> DB<u16, u16, 2, 256> {
    const DIMENSIONS: usize = 2;
    const CAPACITY: usize = 256;
    DB::<T, U, DIMENSIONS, CAPACITY>::read(&[
        c.0[bound_start..bound_start + CAPACITY].to_vec(),
        c.1[bound_start..bound_start + CAPACITY].to_vec(),
    ])
}

fn load_database_2d_reconstructor_final(
    c: (Vec<T>, Vec<T>),
    _bound_start: usize,
) -> DB<u16, u16, 2, 155> {
    const DIMENSIONS: usize = 2;
    const CAPACITY: usize = 256;
    const TOTAL_SIZE: usize = 30363;
    let start = (TOTAL_SIZE / CAPACITY) * CAPACITY;
    DB::<T, U, DIMENSIONS, { TOTAL_SIZE % CAPACITY }>::read(&[
        c.0[start..TOTAL_SIZE].to_vec(),
        c.1[start..TOTAL_SIZE].to_vec(),
    ])
}

fn reconstruct(c: (Vec<T>, Vec<T>), rx: Receiver<[(u16, u16); 2]>) -> JoinHandle<()> {
    thread::spawn(move || {
        const DIMENSIONS: usize = 2;
        const CAPACITY: usize = 256;
        const DOMAINS: [T; DIMENSIONS] = [25457, 24053];
        const QRMW: usize = qrsw(CAPACITY);
        let dummy_qr = Simd::from_array([0usize; QRMW]);
        let mut queries = Vec::new();
        let mut start = 0;
        while start < (30363 / 256) * 256 {
            let db = load_database_2d_reconstructor(c.clone(), start);
            let mut q_index = 0;
            start += 256;

            let mut solver: midline::Tracker<DIMENSIONS, QRMW> =
                midline::Tracker::new::<CAPACITY>(DOMAINS);
            while !<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(
                &solver,
            ) {
                if q_index == queries.len() {
                    if let Ok(q) = rx.recv() {
                        queries.push(q)
                    } else {
                        panic!("Not enough info: {:?}", queries);
                    }
                }
                let query = queries[q_index];
                solver.add_query(&query, dummy_qr, &db);
                q_index += 1;
            }
        }
        let db = load_database_2d_reconstructor_final(c, start);
        let mut q_index = 0;

        let mut solver: midline::Tracker<DIMENSIONS, QRMW> = midline::Tracker::new::<155>(DOMAINS);
        while !<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver)
        {
            if q_index == queries.len() {
                queries.push(rx.recv().unwrap());
            }
            let query = queries[q_index];
            solver.add_query(&query, dummy_qr, &db);
            q_index += 1;
        }
    })
}

fn manager<const WORKERS: usize>() -> u64 {
    let channel = mpmc::channel();
    let load = load_db_columns();
    let db = load_database_2d(load.clone());
    let r = reconstruct(load, channel.1.clone());
    let workers: [_; WORKERS] = core::array::from_fn(|_| {
        spawn_worker_2d(
            channel.0.clone(),
            channel.1.clone(),
            db.0.clone(),
            db.1.clone(),
        )
    });
    drop(channel);
    let mut total = 0;
    for w in workers {
        let queries = w.join().unwrap();
        total += queries;
    }
    println!("Found Queries");
    r.join().unwrap();

    total
}

fn main() {
    println!("Solved database using {} queries", manager::<4>());
}
