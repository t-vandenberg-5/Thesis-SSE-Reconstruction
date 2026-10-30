use reconstructor::{T, dataset::{CSVConfigBuilder, get_minimum, parse}, midline_fast};
use std::thread::{self, JoinHandle};
use nanorand::{Rng, WyRand};

#[macro_export]
macro_rules! query {
    ( $x:expr ) => {
        {
            let mut value: u64 = $x.generate();
            let mut value_index = 0;
            let bounds = [25456, 25455, 24052, 24051];
            let widths = [15, 15, 15, 15];
            let masks = [0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF];
            let mut index = 0;
            let mut raw = [0u16; 4];

            while index < 4 {
                raw[index] = ((value >> value_index) & masks[index]) as T;
                if value_index + widths[index] >= 64 {
                    value = $x.generate();
                    raw[index] |= ((value << (64 - value_index)) & masks[index]) as T;
                }
                value_index = (value_index + widths[index]) % 64;
                index += (raw[index] <= bounds[index]) as usize;
            }

            raw[0] = raw[(raw[0] == 25456) as usize];
            raw[2] = raw[(raw[2] == 24052) as usize];

            [
                (raw[0].min(raw[1]), raw[0].max(raw[1])),
                (raw[2].min(raw[3]), raw[2].max(raw[3])),
            ]
        }
    };
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
        .filter(column3, |x| x != "null")
        ;
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
    const DIMENSIONS_MINUS_ONE: usize = DIMENSIONS - 1;
    const THREADS: usize = 25;
    const TESTS_PER_THREAD: usize = 4;
    let handle: [JoinHandle<[usize; TESTS_PER_THREAD]>; THREADS] = core::array::from_fn(|_threadnum| {
        let v1 = c1.clone();
        let v2 = c2.clone();
        thread::spawn(move || {
            let zipped: [usize; TESTS_PER_THREAD] = core::array::from_fn(|_| {
                let mut sqe = WyRand::new();
                let mut queries = 0;
                let mut solver = midline_fast::Solver::<29252, DIMENSIONS>::new::<DIMENSIONS_MINUS_ONE>(&[v1.clone(), v2.clone()]);
                while !solver.is_solved() {
                    queries += 1;
                    let query = query!(sqe);
                    solver.process::<DIMENSIONS_MINUS_ONE>(query, 0);
                    solver.process::<DIMENSIONS_MINUS_ONE>(query, 1);
                }
                println!("{queries}");

                queries
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
