use super::{T, U};
use core::{assert_eq, iter::Iterator};
use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    simd::num::SimdUint,
};

use crate::{Solver, zero_mask};
use database_engine::{DB, QueryResultMask, filled_mask};

pub struct Tracker<const DIMENSIONS: usize, const QRMW: usize> {
    unsolved_mask: [QueryResultMask<QRMW>; DIMENSIONS],
    solved: HashMap<U, [(T, T); DIMENSIONS]>,
    domain: [T; DIMENSIONS],
    midline: [T; DIMENSIONS],
}

impl<const DIMENSIONS: usize, const QRMW: usize> Tracker<DIMENSIONS, QRMW> {
    pub fn new<const CAPACITY: usize>(domains: [T; DIMENSIONS]) -> Self {
        Self {
            unsolved_mask: [filled_mask::<CAPACITY, QRMW>(); DIMENSIONS],
            solved: HashMap::with_capacity(CAPACITY),
            domain: domains,
            midline: core::array::from_fn(|x| domains[x] / 2),
        }
    }

    pub fn reset_current_area(&mut self, _domains: [T; DIMENSIONS]) {
        panic!("Should not be called when constructing with midlines!")
    }

    pub fn remaining(&self) -> usize {
        let mut answer = 0;
        for d in self.unsolved_mask {
            answer += d.count_ones().as_array().iter().sum::<usize>();
        }
        answer
    }

    pub fn remaining_pos(&self) -> [Vec<usize>; DIMENSIONS] {
        core::array::from_fn(|d| {
            let mut v = Vec::new();
            for (index, block) in (self.unsolved_mask[d]).to_array().iter().enumerate() {
                let mut b = *block;
                while let Some(i) = b.highest_one() {
                    v.push(i as usize + (index * usize::BITS as usize));
                    b -= 1 << i;
                }
            }
            v
        })
    }
}

impl<const DIMENSIONS: usize, const CAPACITY: usize, const QRMW: usize>
    Solver<DIMENSIONS, CAPACITY, QRMW> for Tracker<DIMENSIONS, QRMW>
{
    fn is_solved(&self) -> bool {
        for d in 0..DIMENSIONS {
            if self.unsolved_mask[d] != zero_mask() {
                return false;
            }
        }
        true
    }

    fn get_solved(&self, index: U) -> Option<[T; DIMENSIONS]> {
        match self.solved.get(&index) {
            Some(s) => Some(core::array::from_fn(|d| s[d].0)),
            None => None,
        }
    }

    // TODO: Validate correctness
    fn add_query(
        &mut self,
        query: &[(T, T); DIMENSIONS],
        _query_result: QueryResultMask<QRMW>,
        database: &DB<T, U, DIMENSIONS, CAPACITY>,
    ) -> bool {
        let outside: usize = query
            .iter()
            .enumerate()
            .map(|(d, (s, e))| {
                if *s <= self.midline[d] && self.midline[d] <= *e {
                    0
                } else {
                    1
                }
            })
            .sum();
        if outside <= 1 {
            let query_result = database.search(*query);
            for d in 0..DIMENSIONS {
                if outside == 0
                    || outside == 1
                        && !(query[d].0 <= self.midline[d] && self.midline[d] <= query[d].1)
                {
                    for (index, block) in (query_result & self.unsolved_mask[d])
                        .to_array()
                        .iter()
                        .enumerate()
                    {
                        let mut b = *block;
                        while let Some(i) = b.highest_one() {
                            match self
                                .solved
                                .entry((i as usize + (index * usize::BITS as usize)) as U)
                            {
                                Occupied(mut occupied_entry) => {
                                    let entry = occupied_entry.get_mut();
                                    entry[d].0 = entry[d].0.max(query[d].0);
                                    entry[d].1 = entry[d].1.min(query[d].1);
                                    if entry[d].0 == entry[d].1 {
                                        self.unsolved_mask[d][index] &= !(1 << i);
                                        assert_eq!(
                                            database.cheat_pos_traverse(
                                                (i as usize + (index * usize::BITS as usize)) as U
                                            )[d],
                                            entry[d].0
                                        );
                                    }
                                }
                                Vacant(vacant_entry) => {
                                    let mut base =
                                        core::array::from_fn(|d| (0, self.domain[d] - 1));
                                    base[d] = query[d];
                                    if query[d].0 == query[d].1 {
                                        self.unsolved_mask[d][index] &= !(1 << i);
                                        assert_eq!(
                                            database.cheat_pos_traverse(
                                                (i as usize + (index * usize::BITS as usize)) as U
                                            )[d],
                                            query[d].0
                                        );
                                    }
                                    vacant_entry.insert(base);
                                }
                            }
                            b -= 1 << i;
                        }
                    }
                }
            }
        }
        false
    }
}
