#![feature(portable_simd)]
use bumpalo::Bump;
use database_engine::{DB, qrsw};
use reconstructor::{Solver, T, Tracker2D, Tracker3D, U, midline::{self, Tracker}, traverse::{OnlyMiddle, Traversable, Walk}};
use std::{collections::HashSet, simd::Simd, thread::{self, JoinHandle}};
use emptyblock::simd::simd_block_width;
use nanorand::WyRand;
use query_engine::{buffer::{Buffer, BufferTrait, LoadedBuffer}, generator::UniformRandomGenerator};

fn gen_db(dom: T) -> DB<T, U, DIMENSIONS, CAPACITY> {
    DB::<T, U, DIMENSIONS, CAPACITY>::generate(&[dom; DIMENSIONS])
}

fn get_full_reconstruction_qe<'a>(bump: &'a Bump, dom: T, db: &DB<T, U, DIMENSIONS, CAPACITY>) -> Buffer<'a, T, DIMENSIONS, UniformRandomGenerator<WyRand, 8>> {
    let mut qe = Buffer::<T, DIMENSIONS, UniformRandomGenerator<WyRand, 8>>::new(bump);
    let mut solver: TrackerD = TrackerD::new::<CAPACITY>([dom; DIMENSIONS]);
    while !<TrackerD as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver) {
        let query = qe.next(&[dom; DIMENSIONS]);
        if solver.add_query(&query, db.search(query), &db) {
            qe.restart();
            solver.reset_current_area([dom; DIMENSIONS]);
        }
    }

    for i in 0..CAPACITY {
        assert_eq!(<TrackerD as Solver<DIMENSIONS, CAPACITY, QRMW>>::get_solved(&solver, i as U), Some(db.cheat_pos_traverse(i as U)));
    }

    qe
}

fn find_contained_point(query: [(T, T); DIMENSIONS], insert: &HashSet<[T; DIMENSIONS]>) -> Option<[T; DIMENSIONS]> {
    'p: for p in insert {
        for d in 0..DIMENSIONS {
            if query[d].0 > p[d] || p[d] > query[d].1 {
                continue 'p
            }
        }
        return Some(p.clone())
    }
    None
}

/// Reconstruct the query starting from a record already present in the
/// inserted set. Return value is `true` if the reconstruction succeeded.
fn reconstruct_query_from_point(dom: T, query: [(T, T); DIMENSIONS], inserted: &[T; DIMENSIONS], insert: &mut HashSet<[T; DIMENSIONS]>) -> bool {
    // If the point is outside the query, we have not found the
    // right point yet.
    for d in 0..DIMENSIONS {
        if query[d].0 > inserted[d] || inserted[d] > query[d].1 {
            return false
        }
    }

    // If we found an existing point in the query, expand outward
    // until the boundaries are defined.
    let mut new = inserted.clone();
    for d in 0..DIMENSIONS {
        let min = if query[d].0 > 0 { query[d].0 - 1 } else { query[d].0 };
        let max = if query[d].1 + 1 < dom { query[d].1 + 1 } else { query[d].1 };
        for i in min..=max {
            new[d] = i;
            insert.insert(new);
        }
        new[d] = inserted[d];
    }
    true
}

fn reconstruct_query_from_inserted(dom: T, query: [(T, T); DIMENSIONS], insert: &mut HashSet<[T; DIMENSIONS]>) -> bool {
    if let Some(inserted) = find_contained_point(query, insert) {
        reconstruct_query_from_point(dom, query, &inserted, insert)
    } else {
        false
    }
}

fn n(dom: T, qe: &mut Buffer<T, DIMENSIONS, UniformRandomGenerator<WyRand, 8>>) -> HashSet<[T; DIMENSIONS]> {
    let mut insert: HashSet<[T; DIMENSIONS]> = HashSet::new();
    let mut traverse = OnlyMiddle::<DIMENSIONS>::new(dom);
    let mut traverse_backup = Walk::<DIMENSIONS>::new(dom);

    qe.restart();
    // Loop over all queries
    'q: while let Some(query) = qe.existing_next() {
        if reconstruct_query_from_inserted(dom, query, &mut insert) { continue 'q }

        loop {
            //insert new
            let inserted;
            if let Some(t) = traverse.next() {
                inserted = t;
                insert.insert(t);
            } else if let Some(t) = traverse_backup.next() {
                if !insert.contains(&t) {
                    inserted = t;
                    insert.insert(t);
                } else {
                    continue;
                }
            } else {
                panic!("Already inserted all records, and none were found in query!")
            }

            if reconstruct_query_from_point(dom, query, &inserted, &mut insert) { continue 'q }
        }
    }
    insert
}

fn n2(
    dom: T,
    qe: &mut Buffer<T, DIMENSIONS, UniformRandomGenerator<WyRand, 8>>,
    database: &DB<T, U, DIMENSIONS, CAPACITY>
) -> HashSet<[T; DIMENSIONS]>{
    let mut traverse = OnlyMiddle::new(dom);
    let mut traverse_backup = Walk::new(dom);
    let mut inserted = HashSet::new();
    let mut solver = TrackerD::new::<CAPACITY>([dom; DIMENSIONS]);
    let mut new = Vec::new();
    let bump = Bump::new();
    let mut qe2 = LoadedBuffer::new(&bump);
    let mut qe2_hashset = HashSet::new();
    's: while !<TrackerD as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver) {
        new.clear();
        let record = if let Some(record) = traverse.next() {
            record
        } else if let Some(record) = traverse_backup.next() {
            record
        } else {
            panic!("Added all records: {}", inserted.len());
        };

        if !inserted.contains(&record) {
            inserted.insert(record);
            new.push(record);
        }

        let mut inserted_index = 0;
        while inserted_index < new.len() {
            let point = new[inserted_index];
            qe.restart();
            'q: while let Some(query) = qe.existing_next() {
                if !qe2_hashset.contains(&query) {
                    for d in 0..DIMENSIONS {
                        if query[d].0 > point[d] || point[d] > query[d].1 {
                            continue 'q
                        }
                    }

                    // Found a new query
                    let mut new_point = point.clone();
                    for d in 0..DIMENSIONS {
                        let min = if query[d].0 > 0 { query[d].0 - 1 } else { query[d].0 };
                        let max = if query[d].1 + 1 < dom { query[d].1 + 1 } else { query[d].1 };
                        for i in min..=max {
                            new_point[d] = i;
                            if !inserted.contains(&new_point) {
                                inserted.insert(new_point);
                                new.push(new_point);
                            }
                        }
                        new_point[d] = point[d];
                    }
                    qe2_hashset.insert(query);
                    qe2.load(query);
                    while let Some(pq) = <LoadedBuffer<'_, [(T, T); DIMENSIONS]> as BufferTrait<T, DIMENSIONS>>::existing_next(&mut qe2) {
                        if solver.add_query(&pq, database.search(pq), &database) {
                            <LoadedBuffer<'_, [(T, T); DIMENSIONS]> as BufferTrait<T, DIMENSIONS>>::restart(&mut qe2);
                            solver.reset_current_area([dom; DIMENSIONS]);
                        }
                        if <TrackerD as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver) {
                            break 's;
                        }
                    }
                }
            }
            inserted_index += 1;
        }
    }
    inserted
}

const MIN: T = 10;
const MAX: T = 100;
const AMOUNT: usize = (MAX - MIN + 1) as usize;
const DIMENSIONS: usize = 2;
const CAPACITY: usize = 5;
const DIMENSIONS_MINUS_ONE: usize = DIMENSIONS - 1;
const QRMW: usize = qrsw(CAPACITY);
const THREADS: usize = 25;
const ATTEMPTS_PER_THREAD: usize = 4;
type Tracker2DF = Tracker2D<{ MAX as usize }, QRMW, { simd_block_width(MAX as usize).0 }, { simd_block_width(MAX as usize).1 }>;
type Tracker3DF = Tracker3D<{ MAX as usize }, {MAX as usize }, QRMW, { simd_block_width(MAX as usize).0 }, { simd_block_width(MAX as usize).1 }>;
type TrackerD = Tracker2DF;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut result = ([0usize; AMOUNT], [0usize; AMOUNT], [0usize; AMOUNT]);
    let mut result_inject = ([0usize; AMOUNT], [0usize; AMOUNT], [0usize; AMOUNT]);
    let mut result_inject2 = ([0usize; AMOUNT], [0usize; AMOUNT], [0usize; AMOUNT]);
    let handle: [JoinHandle<([[usize; ATTEMPTS_PER_THREAD]; AMOUNT], [[usize; ATTEMPTS_PER_THREAD]; AMOUNT], [[usize; ATTEMPTS_PER_THREAD]; AMOUNT])>; THREADS] = core::array::from_fn(|_thread_num| thread::spawn(move || {
        let mut dom: T = MIN;
        let mut result = [[0usize; ATTEMPTS_PER_THREAD]; AMOUNT];
        let mut result_insert = [[0usize; ATTEMPTS_PER_THREAD]; AMOUNT];
        let mut result_insert2 = [[0usize; ATTEMPTS_PER_THREAD]; AMOUNT];
        while dom <= MAX {
            let zipped: [(usize, usize, usize); ATTEMPTS_PER_THREAD] = core::array::from_fn(|_| {
                let db = gen_db(dom);
                let bump = Bump::new();
                let mut qe = get_full_reconstruction_qe(&bump, dom, &db);
                let insert = n(dom, &mut qe);
                let insert_2 = n2(dom, &mut qe, &db);

                (qe.size(), insert.len(), insert_2.len())
            });
            for (i, (r, r_i, r_i2)) in zipped.iter().enumerate() {
                (result[(dom - MIN) as usize][i], result_insert[(dom - MIN) as usize][i], result_insert2[(dom - MIN) as usize][i]) = (*r, *r_i, *r_i2);
            }
            dom += 1;
        }
        (result, result_insert, result_insert2)
    }));

    let mut r: [Vec<usize>; AMOUNT] = core::array::from_fn(|_| Vec::with_capacity(THREADS * ATTEMPTS_PER_THREAD));
    let mut r_i: [Vec<usize>; AMOUNT] = core::array::from_fn(|_| Vec::with_capacity(THREADS * ATTEMPTS_PER_THREAD));
    let mut r_i2: [Vec<usize>; AMOUNT] = core::array::from_fn(|_| Vec::with_capacity(THREADS * ATTEMPTS_PER_THREAD));

    for h in handle {
        let data = h.join().unwrap();
        for i in 0..r.len() {
            r[i].extend_from_slice(&data.0[i]);
            r_i[i].extend_from_slice(&data.1[i]);
            r_i2[i].extend_from_slice(&data.2[i]);
        }
    }

    for i in MIN..=MAX {
        let index = (i-MIN) as usize;
        r[index].sort();
        r_i[index].sort();
        r_i2[index].sort();
        (result.0[index], result.1[index], result.2[index]) = (r[index][50], r[index][90], r[index][99]);
        (result_inject.0[index], result_inject.1[index], result_inject.2[index]) = (r_i[index][50], r_i[index][90], r_i[index][99]);
        (result_inject2.0[index], result_inject2.1[index], result_inject2.2[index]) = (r_i2[index][50], r_i2[index][90], r_i2[index][99]);
    }
    println!("full {:?}", result);
    println!("inject {:?}", result_inject);
    println!("inject2 {:?}", result_inject2);

    let dummy_qr = Simd::from_array([0usize; QRMW]);
    let mut result_midline = ([0usize; AMOUNT], [0usize; AMOUNT], [0usize; AMOUNT]);
    let mut result_midline_inject = ([0usize; AMOUNT], [0usize; AMOUNT], [0usize; AMOUNT]);
    let mut result_midline_inject2 = ([0usize; AMOUNT], [0usize; AMOUNT], [0usize; AMOUNT]);
    let handle: [JoinHandle<([[usize; ATTEMPTS_PER_THREAD]; AMOUNT], [[usize; ATTEMPTS_PER_THREAD]; AMOUNT], [[usize; ATTEMPTS_PER_THREAD]; AMOUNT])>; THREADS] = core::array::from_fn(|_| thread::spawn(move || {
        let mut dom: T = MIN;
        let mut result = [[0usize; ATTEMPTS_PER_THREAD]; AMOUNT];
        let mut result_insert = [[0usize; ATTEMPTS_PER_THREAD]; AMOUNT];
        let mut result_insert_2 = [[0usize; ATTEMPTS_PER_THREAD]; AMOUNT];

        while dom <= MAX {
            let dom_arr = [dom; DIMENSIONS];
            let midpoint_arr = [dom / 2; DIMENSIONS];
            let zipped: [(usize, usize, usize); ATTEMPTS_PER_THREAD] = core::array::from_fn(|_| {
                let database = DB::<T, U, DIMENSIONS, CAPACITY>::generate(&dom_arr);
                let bump = Bump::new();
                let mut qe = Buffer::<T, DIMENSIONS, UniformRandomGenerator<WyRand, 8>>::new(&bump);
                let insert: HashSet<[T; DIMENSIONS]>;
                //c.bench_function("simd_gen_query", |b| b.iter(|| database.generate_query()));
                let mut queries = 0;
                let mut solver: midline::Tracker::<DIMENSIONS, QRMW> = midline::Tracker::new::<CAPACITY>(dom_arr);

                if cfg!(debug_assertions) {
                    use reconstructor::midline_fast;

                    let mut data = core::array::from_fn(|_| Vec::with_capacity(CAPACITY));
                    for r in 0..CAPACITY {
                        let pos = database.cheat_pos_traverse(r as U);
                        for d in 0..DIMENSIONS {
                            data[d].push(pos[d]);
                        }
                    }
                    let mut new_solver = midline_fast::Solver::<CAPACITY, DIMENSIONS>::new::<DIMENSIONS_MINUS_ONE>(&data);

                    while !<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver) {
                        let query = qe.next(&dom_arr);

                        queries += 1;
                        solver.add_query(&query, dummy_qr, &database);
                        //println!("Added query to old");
                        if !new_solver.is_solved() {
                            new_solver.process_midline::<DIMENSIONS_MINUS_ONE>(query, &midpoint_arr);
                            // println!("Added query to new");
                            if new_solver.is_solved() {
                                if <Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver) {
                                    println!("SOLVED: {}", queries);
                                } else {
                                    if queries <= 50 {
                                        for r in 0..CAPACITY {
                                            println!("{:?}", database.cheat_pos_traverse(r as T));
                                        }

                                        qe.restart();
                                        for _ in 0..queries {
                                            println!("{:?}", qe.existing_next().unwrap());
                                        }
                                        assert_eq!(qe.existing_next(), None);
                                        panic!();

                                    }
                                    panic!("PRESOLVED: {}", queries);
                                }
                            }
                        } else {
                            panic!("{}", new_solver.remaining())
                        }
                    }
                    if !new_solver.is_solved() {
                        println!("{:?}", new_solver.remaining());
                    }
                    assert!(new_solver.is_solved());
                } else {
                    while !<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::is_solved(&solver) {
                        let query = qe.next(&dom_arr);

                        queries += 1;
                        solver.add_query(&query, dummy_qr, &database);
                    }
                }

                insert = n(dom, &mut qe);
                let insert_2 = n2(dom, &mut qe, &database);


                for i in 0..CAPACITY {
                    assert_eq!(<Tracker<DIMENSIONS, QRMW> as Solver<DIMENSIONS, CAPACITY, QRMW>>::get_solved(&solver, i as U), Some(database.cheat_pos_traverse(i as U)));
                }
                (queries as usize, insert.len(), insert_2.len())
            });
            for (i, (r, r_i, r_i2)) in zipped.iter().enumerate() {
                (result[(dom - MIN) as usize][i], result_insert[(dom - MIN) as usize][i], result_insert_2[(dom - MIN) as usize][i]) = (*r, *r_i, *r_i2);
            }
            dom += 1;
        }
        (result, result_insert, result_insert_2)
    }));

    let mut r: [Vec<usize>; AMOUNT] = core::array::from_fn(|_| Vec::with_capacity(THREADS * ATTEMPTS_PER_THREAD));
    let mut r_i: [Vec<usize>; AMOUNT] = core::array::from_fn(|_| Vec::with_capacity(THREADS * ATTEMPTS_PER_THREAD));
    let mut r_i2: [Vec<usize>; AMOUNT] = core::array::from_fn(|_| Vec::with_capacity(THREADS * ATTEMPTS_PER_THREAD));

    for h in handle {
        let data = h.join().unwrap();
        for i in 0..r.len() {
            r[i].extend_from_slice(&data.0[i]);
            r_i[i].extend_from_slice(&data.1[i]);
            r_i2[i].extend_from_slice(&data.2[i]);
        }
    }

    for i in MIN..=MAX {
        let index = (i-MIN) as usize;
        r[index].sort();
        r_i[index].sort();
        r_i2[index].sort();
        (result_midline.0[index], result_midline.1[index], result_midline.2[index]) = (r[index][50], r[index][90], r[index][99]);
        (result_midline_inject.0[index], result_midline_inject.1[index], result_midline_inject.2[index]) = (r_i[index][50], r_i[index][90], r_i[index][99]);
        (result_midline_inject2.0[index], result_midline_inject2.1[index], result_midline_inject2.2[index]) = (r_i2[index][50], r_i2[index][90], r_i2[index][99]);
    }
    println!("midline {:?}", result_midline);
    println!("midline inject {:?}", result_midline_inject);
    println!("midline inject 2 {:?}", result_midline_inject2);

    let mut d = 0;
    println!("p50");
    println!("f,fi,fi2,m,mi,mi2");
    while d < result.0.len() {
        println!("{},{},{},{},{},{}", result.0[d], result_inject.0[d], result_inject2.0[d], result_midline.0[d], result_midline_inject.0[d], result_midline_inject2.0[d]);
        d += 1;
    }
    println!("p90");
    println!("f,fi,fi2,m,mi,mi2");
    d = 0;
    while d < result.1.len() {
        println!("{},{},{},{},{},{}", result.1[d], result_inject.1[d], result_inject2.1[d], result_midline.1[d], result_midline_inject.1[d], result_midline_inject2.1[d]);
        d += 1;
    }
    println!("p99");
    println!("f,fi,fi2,m,mi,mi2");
    d = 0;
    while d < result.2.len() {
        println!("{},{},{},{},{},{}", result.2[d], result_inject.2[d], result_inject2.2[d], result_midline.2[d], result_midline_inject.2[d], result_midline_inject2.2[d]);
        d += 1;
    }
    Ok(())

    // let name = format!("{}-{}.svg", MIN, MAX);
    // let range = MIN..MAX;
    // let mut chart = QuadChart::new(&name, (MIN as f32)..(MAX as f32))?;
    // chart.draw_line(&range, &result, BLACK, "3D: Full reconstruction".to_string())?;
    // chart.draw_line(&range, &result_inject, GREEN, "3D: Full reconstruction injected".to_string())?;
    // chart.draw_line(&range, &result_inject2, ORANGE, "3D: Full reconstruction injected 2".to_string())?;
    //
    // chart.draw_line(&range, &result_midline, RED, "3D: Midline reconstruction".to_string())?;
    // chart.draw_line(&range, &result_midline_inject, PURPLE, "3D: Midline reconstruction injected".to_string())?;
    // chart.draw_line(&range, &result_midline_inject2, YELLOW, "3D: Midline reconstruction injected 2".to_string())?;
    //
    // let v: [_; AMOUNT] = core::array::from_fn(|x| (((x + MIN as usize) as f32)*(((x + MIN as usize) as f32).ln())) as usize);
    // let all = (v, v, v);
    //
    // chart.draw_line(&range, &all, CYAN, "3D: All".to_string())?;
    //
    // Ok(chart.draw()?)
}
