use std::cmp::Ordering;

// The general idea behind this approach is that instead of running queries and 
// analyzing the results, we should simply parse the database into another
// datastructure that sets requirements. As these requirements are met, we can
// remove them. Once none remain, the database is reconstructed.
use crate::{T, midline_optimized::pebble::Pebble};

pub const fn hyperoctants(m: usize) -> usize { 2usize.pow(m as u32) }

/// Contains a set of constraints
mod raw_pebble {
    use std::cmp::Ordering;

    use crate::T;

    fn flip_order(order: Ordering) -> Ordering {
        match order {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct RawPebble<const N: usize> {
        pos: Vec<T>,
        constraints: Vec<[T; N]>,
    }

    impl<const N: usize> RawPebble<N> {
        pub fn with_capacity(capacity: usize) -> Self {
            Self { pos: Vec::with_capacity(capacity), constraints: Vec::with_capacity(capacity) }
        }

        pub fn insert<const M: usize>(&mut self, index: usize, value: &[T; M]) {
            debug_assert_eq!(N + 1, M);
            debug_assert!(M > N);
            debug_assert_eq!(self.pos.len(), self.constraints.len());
            self.pos.insert(index, value[0]);
            self.constraints.insert(index, core::array::from_fn(|x| value[x + 1]));
        }

        pub fn replace<const M: usize>(&mut self, index: usize, value: &[T; M]) {
            debug_assert_eq!(N + 1, M);
            debug_assert!(M > N);
            debug_assert_eq!(self.pos.len(), self.constraints.len());
            self.pos[index] = value[0];
            self.constraints[index] = core::array::from_fn(|x| value[x + 1]);
        }

        pub fn get(&self, index: usize) -> (&T, &[T; N]) {
            debug_assert_eq!(self.pos.len(), self.constraints.len());
            (&self.pos[index], &self.constraints[index])
        }

        pub fn remove(&mut self, index: usize) {
            debug_assert_eq!(self.pos.len(), self.constraints.len());
            self.pos.remove(index);
            self.constraints.remove(index);
        }

        pub fn len(&self) -> usize {
            debug_assert_eq!(self.pos.len(), self.constraints.len());
            self.pos.len()
        }

        pub fn is_solved(&self) -> bool {
            debug_assert_eq!(self.pos.len(), self.constraints.len());
            self.pos.is_empty()
        }

        pub fn binary_search(&self, value: T) -> Result<usize, usize> {
            debug_assert_eq!(self.pos.len(), self.constraints.len());
            self.pos.binary_search(&value)
        }

        /// Check a record's dominance relation to an entry in Pebble.
        /// 
        /// It returns `Some(Greater)` if the record is dominant to the entry, 
        /// `Some(Equal)` if the record is equal to the entry, `Some(Less)` if the
        /// record is dominated by the entry, and None if there is no dominance
        /// between the record and entry.
        pub fn cmp<const M: usize>(
            &self,
            index: usize, 
            record: &[T; M], 
            hyperoctant_index: usize, 
        ) -> Option<Ordering> {
            debug_assert_eq!(N + 1, M);
            debug_assert!(M > N);

            //println!("called cmp, {:?}, {}, {:?}, {}", self, index, record, hyperoctant_index);

            let mut order = Ordering::Equal;
            let (pos, constraints) = self.get(index);
            debug_assert_eq!(record[0], *pos);

            for d in 0..N {
                let entry_cmp = 
                    if (1 << d) & hyperoctant_index > 0 { 
                        record[d + 1].cmp(&constraints[d])
                    } else { 
                        flip_order(record[d + 1].cmp(&constraints[d]))
                    };

                match (order, entry_cmp) {
                    (Ordering::Less, Ordering::Less) |
                    (Ordering::Equal, Ordering::Equal) |
                    (Ordering::Greater, Ordering::Greater)
                        => {},
                    (_, Ordering::Equal)
                        => {},
                    (Ordering::Equal, a) 
                        => { order = a },
                    (Ordering::Less, Ordering::Greater) |
                    (Ordering::Greater, Ordering::Less) 
                        => { return None },
                }
            }
            // println!("Order: {:?}", order);
            Some(order)
        }

        /// Assumes the query is valid for the current dimension, has no way to 
        /// check. Furthermore, assumes that the correct query bound was used to 
        /// find the index.
        fn resolves(
            &self,
            index: usize,
            query: &[(T, T)], 
            hyperoctant_index: usize, 
        ) -> bool {
            debug_assert_eq!(N, query.len());

            let (_pos, constraints) = self.get(index);

            // println!("resolves: self: {:?}, index: {}, query: {:?}, hyperoctant_index: {}", self.get(index), index, query, hyperoctant_index);

            for d in 0..N {
                // println!("d: {}", d);
                if (1 << d) & hyperoctant_index > 0 { 
                    // println!("if");
                    if query[d].1 < constraints[d] {
                        return false;
                    }
                } else { 
                    // println!("else");
                    // println!("qd0: {}, c: {}", query[d].0, constraints[d]);
                    if query[d].0 > constraints[d] {
                        return false;
                    }
                }
            }
            // println!("true");
            true
        }

        pub fn process_query<const M: usize>(
            &mut self,
            query: &[(T, T); M], 
            hyperoctant_index: usize, 
            is_upper: bool,
        ) {
            debug_assert_eq!(N + 1, M);
            debug_assert!(M > N);

            let to_match = if is_upper { query[0].1 } else { query[0].0 };

            // println!("A: query: {:?}, hyperoctant_index: {}, is_upper: {}, to_match: {}", query, hyperoctant_index, is_upper, to_match);
            // println!("{:?}, {:?}", self.pos, self.binary_search(to_match));

            if let Ok(index) = self.binary_search(to_match) {
                let mut higher = index + 1;
                if self.resolves(index, &query[1..], hyperoctant_index) {
                    self.remove(index);
                    higher -= 1;
                }

                while higher < self.len() && self.pos[higher] == to_match {
                    if self.resolves(higher, &query[1..], hyperoctant_index) {
                        self.remove(higher);
                    } else {
                        higher += 1;
                    }
                }

                let mut lower = index.wrapping_sub(1);
                while lower < usize::MAX && self.pos[lower] == to_match {
                    if self.resolves(lower, &query[1..], hyperoctant_index) {
                        self.remove(lower);
                    }
                    lower = lower.wrapping_sub(1);
                }
            }

        }
    }
}

mod pebble {
    use std::cmp::Ordering;

    use crate::{T, midline_optimized::raw_pebble::RawPebble};

    #[derive(Clone, Debug)]
    pub struct Pebble<const N: usize>(RawPebble<N>);

    impl<const N: usize> Pebble<N> {
        pub fn with_capacity(capacity: usize) -> Self {
            Self(RawPebble::with_capacity(capacity))
        }

        pub fn is_solved(&self) -> bool {
            self.0.is_solved()
        }

        pub fn insert<const M: usize>(&mut self, record: &[T; M], hyperoctant_index: usize) {
            debug_assert_eq!(N + 1, M);
            debug_assert!(M > N);

            match self.0.binary_search(record[0]) {
                Ok(index) => {
                    if let Some(order) = self.0.cmp(index, record, hyperoctant_index) {
                        if order == Ordering::Greater {
                            // println!("Called replace!");
                            self.0.replace(index, record);
                        }
                        return 
                    }

                    let mut lower = index.wrapping_sub(1);
                    while lower < usize::MAX && *self.0.get(lower).0 == record[0] {
                        if let Some(order) = self.0.cmp(lower, record, hyperoctant_index) {
                            if order == Ordering::Greater {
                                // println!("Called replace!");
                                self.0.replace(lower, record);
                            }
                            return 
                        }

                        lower = lower.wrapping_sub(1);
                    }

                    let mut higher = index + 1;
                    while higher < self.0.len() && *self.0.get(higher).0 == record[0] {
                        if let Some(order) = self.0.cmp(higher, record, hyperoctant_index) {
                            if order == Ordering::Greater {
                                // println!("Called replace!");
                                self.0.replace(higher, record);
                            }
                            return 
                        }

                        higher += 1;
                    }

                    self.0.insert(index, record);
                },
                Err(index) => {
                    self.0.insert(index, record);
                },
            }
        }

        pub fn process_query<const M: usize>(
            &mut self, 
            query: &[(T, T); M], 
            hyperoctant_index: usize, 
            is_upper: bool,
        ) {
            debug_assert_eq!(N + 1, M);
            debug_assert!(M > N);

            self.0.process_query(query, hyperoctant_index, is_upper);
        }

        pub fn len(&self) -> usize {
            self.0.len()
        }
    }
}

enum GuardianPebble {
    Lower,
    Upper,
}

/** Contains both upper and lower constraints
 */
#[derive(Clone, Debug)]
struct Guardian<const N: usize> {
    // These differ not in their creation, but in their destruction
    lower: Pebble<N>,
    upper: Pebble<N>,
}

impl<const N: usize> Guardian<N> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self { lower: Pebble::with_capacity(capacity), upper: Pebble::with_capacity(capacity) }
    }

    pub fn is_solved(&self) -> bool {
        self.lower.is_solved() && self.upper.is_solved()
    }

    pub fn insert<const M: usize>(&mut self, record: &[T; M], hyperoctant_index: usize, guardian_pebble: GuardianPebble) {
        debug_assert_eq!(N + 1, M);
        debug_assert!(M > N);

        match guardian_pebble {
            GuardianPebble::Lower => {
                // println!("Inserting in lower hyperoctant {:?}", record);
                self.lower.insert(record, hyperoctant_index);
            },
            GuardianPebble::Upper => {
                // println!("Inserting in upper hyperoctant {:?}", record);
                self.upper.insert(record, hyperoctant_index);
            },
        }
    }

    pub fn process_query<const M: usize>(
        &mut self, 
        query: &[(T, T); M], 
        hyperoctant_index: usize, 
        guardian_pebble: GuardianPebble
    ) {
        debug_assert_eq!(N + 1, M);
        debug_assert!(M > N);

        match guardian_pebble {
            GuardianPebble::Lower => {
                self.lower.process_query(query, hyperoctant_index, false);
            },
            GuardianPebble::Upper => {
                self.upper.process_query(query, hyperoctant_index, true);
            },
        }
    }

    pub fn len(&self) -> usize {
        self.upper.len() + self.lower.len()
    }
}

/** Contains constraints of all hyperoctants of a single dimension
 */
#[derive(Clone, Debug)]
struct Basalt<const N: usize, const H: usize> {
    // These differ in their creation and destruction
    hyperoctant: [Guardian<N>; H],
}

impl<const N: usize, const H: usize> Basalt<N, H> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self { hyperoctant: core::array::from_fn(|_| Guardian::with_capacity(capacity)) }
    }

    fn find_record_hyperoctant<const M: usize>(record: &[T; M], midpoint: &[T; M]) -> (usize, usize) {
        debug_assert_eq!(N + 1, M);
        debug_assert!(M > N);
        debug_assert_eq!(H, hyperoctants(N));

        let mut lower_hyperoctant = 0;
        let mut upper_hyperoctant = 0;

        for d in 0..N {
            match record[d + 1].cmp(&midpoint[d + 1]) {
                Ordering::Less => {
                    // println!("Less");
                },
                Ordering::Equal => {
                    // println!("Equal");
                    upper_hyperoctant |= 1 << d;
                },
                Ordering::Greater => {
                    // println!("Greater");
                    lower_hyperoctant |= 1 << d;
                    upper_hyperoctant |= 1 << d;
                },
            }
        }

        // println!("Record hyperoctant {}, {}", lower_hyperoctant, upper_hyperoctant);
        (lower_hyperoctant, upper_hyperoctant)
    }

    fn find_query_hyperoctant<const M: usize>(query: &[(T, T); M], midpoint: &[T; M]) -> (usize, usize) {
        debug_assert_eq!(N + 1, M);
        debug_assert!(M > N);
        debug_assert_eq!(H, hyperoctants(N));

        let mut lower_hyperoctant = 0;
        let mut upper_hyperoctant = 0;

        for d in 0..N {
            match query[d + 1].0.cmp(&midpoint[d + 1]) {
                Ordering::Less | Ordering::Equal => {},
                Ordering::Greater => {
                    lower_hyperoctant |= 1 << d;
                },
            }
            match query[d + 1].1.cmp(&midpoint[d + 1]) {
                Ordering::Less => {},
                Ordering::Equal | Ordering::Greater => {
                    upper_hyperoctant |= 1 << d;
                },
            }
        }

        (lower_hyperoctant, upper_hyperoctant)
    }

    pub fn insert<const M: usize>(&mut self, record: &[T; M], midpoint: &[T; M]) {
        debug_assert_eq!(N + 1, M);
        debug_assert!(M > N);
        debug_assert_eq!(H, hyperoctants(N));

        let (lower, upper) = Self::find_record_hyperoctant(record, midpoint);
        // println!("Inserting in hyperoctants {}, {}", lower, upper);
        self.hyperoctant[lower].insert(record, lower, GuardianPebble::Lower);
        self.hyperoctant[upper].insert(record, upper, GuardianPebble::Upper);
    }

    pub fn process_query<const M: usize>(&mut self, query: &[(T, T); M], midpoint: &[T; M]) {
        let (lower, upper) = Self::find_query_hyperoctant(query, midpoint);
        self.hyperoctant[lower].process_query(query, lower, GuardianPebble::Lower);
        self.hyperoctant[lower].process_query(query, lower, GuardianPebble::Upper);
        self.hyperoctant[upper].process_query(query, upper, GuardianPebble::Lower);
        self.hyperoctant[upper].process_query(query, upper, GuardianPebble::Upper);
    }

    pub fn is_solved(&self) -> bool {
        for h in 0..H {
            if !self.hyperoctant[h].is_solved() {
                return false
            }
        }
        true
    }

    pub fn len(&self) -> usize {
        self.hyperoctant.iter().map(|x| x.len()).sum()
    }
}

#[derive(Clone, Debug)]
pub struct Slate<const N: usize, const M: usize, const H: usize> {
    dimensions: [Basalt<N, H>; M],
}

impl<const N: usize, const M: usize, const H: usize> Slate<N, M, H> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self { dimensions: core::array::from_fn(|_| Basalt::with_capacity(capacity)) }
    }

    pub fn is_solved(&self) -> bool {
        for d in 0..M {
            if !self.dimensions[d].is_solved() {
                return false
            }
        }
        true
    }

    pub fn insert(&mut self, record: &mut [T; M], midpoint: &mut [T; M]) {
        for d in 0..M {
            record.swap(0, d);
            midpoint.swap(0, d);

            // println!("Inserting in dimension {}", d);
            self.dimensions[d].insert(record, midpoint);

            record.swap(0, d);
            midpoint.swap(0, d);
        }
    }

    pub fn process_query(&mut self, query: &mut [(T, T); M], midpoint: &mut [T; M]) -> bool {
        let mut outside = 0;
        let mut outside_dimension = 0;
        for d in 0..M {
            if query[d].0 > midpoint[d] || midpoint[d] > query[d].1 {
                outside += 1;
                outside_dimension = d;
            }
        }
        // println!("Query validity: {}, {}", outside, outside_dimension);

        if outside == 0 {
            let start = self.len();
            for d in 0..M {
                query.swap(0, d);
                midpoint.swap(0, d);

                self.dimensions[d].process_query(query, midpoint);

                query.swap(0, d);
                midpoint.swap(0, d);
            }
            self.len() != start
        } else if outside == 1 {
            let start = self.len();
            // println!("{:?}, {:?}", query, midpoint);
            query.swap(0, outside_dimension);
            midpoint.swap(0, outside_dimension);
            // println!("{:?}, {:?}", query, midpoint);

            self.dimensions[outside_dimension].process_query(query, midpoint);

            // println!("{:?}, {:?}", query, midpoint);
            query.swap(0, outside_dimension);
            midpoint.swap(0, outside_dimension);
            // println!("{:?}, {:?}", query, midpoint);
            self.len() != start
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.dimensions.iter().map(|x| x.len()).sum()
    }
}

#[test]
fn testcase() {
    // [6, 6]
    // [6, 6]
    // [(9, 9), (0, 8)]
    // [(5, 9), (5, 6)]
    // [(2, 6), (1, 8)]
    // [(0, 6), (6, 7)]
    // [(3, 9), (1, 5)]
    // [(2, 3), (1, 2)]
    // [(1, 1), (1, 5)]
    // [(0, 6), (3, 7)]
    // [(3, 5), (4, 6)]
    // [(1, 3), (2, 9)]
    // [(0, 2), (9, 9)]
    // [(1, 5), (1, 6)]
    // [(6, 8), (4, 4)]
    // [(6, 7), (4, 5)]
    let mut t = Slate::<1, 2, 2>::with_capacity(2);
    t.insert(&mut [6, 6], &mut [5, 5]);
    t.insert(&mut [6, 6], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 0);
    t.process_query(&mut [(5, 9), (5, 6)], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 0);
    t.process_query(&mut [(2, 6), (1, 8)], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 0);
    t.process_query(&mut [(0, 6), (6, 7)], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 0);
    // println!("Processing Query");
    t.process_query(&mut [(6, 7), (4, 5)], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 0);
}

#[test]
fn testcase2() {
    // [5, 5]
    // [0, 5]
    // [(1, 7), (5, 7)]
    // [(2, 2), (2, 6)]
    // [(2, 7), (4, 8)]
    // [(1, 4), (3, 5)]
    // [(7, 7), (7, 8)]
    // [(2, 3), (5, 5)]
    // [(6, 8), (3, 3)]
    // [(6, 6), (0, 4)]
    // [(1, 7), (6, 7)]
    // [(0, 9), (4, 5)]
    // [(9, 9), (5, 6)]
    // [(3, 7), (9, 9)]
    // [(5, 5), (5, 9)]
    // [(2, 7), (1, 1)]
    // [(4, 4), (4, 6)]
    // [(4, 5), (4, 4)]
    // [(7, 7), (1, 5)]
    // [(0, 6), (7, 7)]
    // [(3, 9), (2, 3)]
    // [(5, 7), (3, 5)]
    // [(1, 2), (2, 5)]
    // [(4, 5), (1, 3)]
    // [(6, 6), (1, 7)]
    // [(0, 0), (0, 5)]
    let mut t = Slate::<1, 2, 2>::with_capacity(2);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
    t.insert(&mut [5, 5], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
    t.insert(&mut [0, 5], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 2);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 2);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
    t.process_query(&mut [(1, 7), (5, 7)], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 2);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 2);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
}

#[test]
fn testcase3() {
    // [3, 5]
    // [3, 1]
    // [(0, 8), (4, 7)]
    // [(0, 3), (2, 3)]
    // [(6, 6), (5, 9)]
    // [(1, 8), (6, 6)]
    // [(2, 3), (5, 5)]
    // [(2, 6), (0, 1)]
    // [(0, 6), (4, 9)]
    // [(0, 7), (0, 1)]
    // [(4, 6), (7, 8)]
    // [(4, 8), (0, 7)]
    // [(0, 3), (3, 8)]
    // [(5, 5), (0, 8)]
    // [(3, 9), (0, 5)]
    // [(2, 6), (7, 8)]
    // [(0, 1), (0, 7)]
    // [(3, 8), (4, 9)]
    // [(1, 7), (7, 9)]
    // [(2, 9), (0, 4)]
    // [(2, 8), (5, 7)]
    // [(8, 9), (0, 3)]
    // [(0, 5), (6, 6)]
    // [(8, 9), (8, 8)]
    // [(5, 7), (1, 8)]
    // [(0, 3), (4, 8)]
    // [(1, 9), (2, 3)]
    // [(6, 9), (1, 4)]
    // [(5, 8), (2, 9)]
    // [(2, 8), (7, 7)]
    // [(0, 8), (2, 7)]
    // [(2, 5), (4, 6)]
    // [(0, 1), (4, 6)]
    // [(8, 8), (4, 8)]
    // [(0, 1), (3, 6)]
    // [(2, 6), (4, 7)]
    // [(7, 7), (1, 1)]
    // [(0, 7), (0, 2)]
    // [(2, 3), (5, 8)]
    // [(1, 6), (2, 6)]
    // [(0, 5), (4, 9)]
    // [(0, 8), (3, 5)]
    // [(7, 8), (3, 6)]
    // [(3, 4), (4, 7)]
    // [(5, 7), (3, 4)]
    // [(8, 9), (0, 4)]
    // [(0, 3), (2, 7)]
    // [(1, 7), (5, 5)]
    // [(1, 9), (1, 5)]
    let mut t = Slate::<1, 2, 2>::with_capacity(2);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
    t.insert(&mut [3, 5], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 1);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
    t.insert(&mut [3, 1], &mut [5, 5]);
    t.process_query(&mut [(1, 7), (5, 7)], &mut [5, 5]);
    assert_eq!(t.dimensions[0].hyperoctant[1].upper.len(), 2);
    assert_eq!(t.dimensions[0].hyperoctant[1].lower.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].upper.len(), 0);
    assert_eq!(t.dimensions[0].hyperoctant[0].lower.len(), 2);
    assert_eq!(t.dimensions[1].hyperoctant[0].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[0].lower.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].upper.len(), 1);
    assert_eq!(t.dimensions[1].hyperoctant[1].lower.len(), 0);
}
