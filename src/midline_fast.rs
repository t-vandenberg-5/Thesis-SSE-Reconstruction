use super::{U, T};

pub struct Solver<const CAPACITY: usize, const DIMENSIONS: usize>{
    dim: [SolverDimension<CAPACITY, DIMENSIONS>; DIMENSIONS],
}

impl<const CAPACITY: usize, const DIMENSIONS: usize> Solver<CAPACITY, DIMENSIONS> {
    pub fn new<const DIMENSIONS_MINUS_ONE: usize>(data: &[Vec<T>; DIMENSIONS]) -> Self {
        Self {
            dim: core::array::from_fn(|d| SolverDimension::new::<DIMENSIONS_MINUS_ONE>(&data[d], &core::array::from_fn(|d2| {
                let i = d2 + (d2 >= d) as usize;
                data[i].clone()
            }))),
        }
    }

    pub fn is_solved(&self) -> bool {
        let mut res = true;
        for d in 0..DIMENSIONS {
            res &= self.dim[d].is_solved();
        }
        res
    }

    pub fn remaining(&self) -> usize {
        let mut v = 0;
        for d in 0..DIMENSIONS {
            v += self.dim[d].remaining();
        }
        v
    }

    pub fn process<const DIMENSIONS_MINUS_ONE: usize>(&mut self, query: [(T, T); DIMENSIONS], d: usize) -> bool {
        let mut return_val = false;
        return_val |= self.dim[d].process::<DIMENSIONS_MINUS_ONE>(query, d);
        return_val
    }

    pub fn process_midline<const DIMENSIONS_MINUS_ONE: usize>(&mut self, query: [(T, T); DIMENSIONS], midpoints: &[T; DIMENSIONS]) {
        let mut outside = 0;
        let mut outside_dimension = 0;
        for d in 0..DIMENSIONS {
            if query[d].0 > midpoints[d] || midpoints[d] > query[d].1 {
                outside += 1;
                outside_dimension = d;
            }
        }

        if outside == 1 {
            self.process::<DIMENSIONS_MINUS_ONE>(query, outside_dimension);
        } else if outside == 0 {
            for d in 0..DIMENSIONS {
                self.process::<DIMENSIONS_MINUS_ONE>(query, d);
            }
        }
    }
}

pub struct SolverDimension<const CAPACITY: usize, const DIMENSIONS: usize>{
    rearranged: Vec<[T; DIMENSIONS]>,
    solved: Vec<u8>, // Supports DIMENSIONS up to and including 5.
}

impl<const CAPACITY: usize, const DIMENSIONS: usize> SolverDimension<CAPACITY, DIMENSIONS> {
    pub fn new<const DIMENSIONS_MINUS_ONE: usize>(primary_dimension: &Vec<T>, other_dimensions: &[Vec<T>; DIMENSIONS_MINUS_ONE]) -> Self {
        assert_eq!(DIMENSIONS_MINUS_ONE + 1, DIMENSIONS);
        assert!(DIMENSIONS_MINUS_ONE < DIMENSIONS);
        Self {
            rearranged: {
                let mut unsorted = core::array::from_fn::<_, CAPACITY, _>(|c| {
                    core::array::from_fn(|d| if d == 0 { primary_dimension[c] } else { other_dimensions[d-1][c] })
                });
                unsorted.sort_by_key(|arr| arr[0]);
                unsorted.to_vec()
            },
            solved: core::array::repeat::<_, CAPACITY>(0).to_vec(),
        }
    }

    pub fn is_solved(&self) -> bool { self.rearranged.is_empty() }

    pub fn remaining(&self) -> usize { self.rearranged.len() }

    pub fn process<const DIMENSIONS_MINUS_ONE: usize>(&mut self, query: [(T, T); DIMENSIONS], d: usize) -> bool {
        let mut return_val = false;
        if let Ok(mut index) = self.rearranged.binary_search_by_key(&query[d].0, |arr| arr[0]) {
            while index > 0 && self.rearranged[index - 1][0] == query[d].0 {
                index -= 1;
            }

            while index < self.rearranged.len() && self.rearranged[index][0] == query[d].0 {
                let mut constrained = true;
                for d2 in 0..DIMENSIONS_MINUS_ONE {
                    constrained &= query[d2 + (d2 >= d) as usize].0 <= self.rearranged[index][d2+1] && self.rearranged[index][d2+1] <= query[d2 + (d2 >= d) as usize].1
                }
                return_val = true;
                self.solved[index] |= (constrained as u8);
                if self.solved[index] == 0b11 {
                    self.rearranged.remove(index);
                    self.solved.remove(index);
                } else {
                    index += 1;
                }
            }
        }

        if let Ok(mut index) = self.rearranged.binary_search_by_key(&query[d].1, |arr| arr[0]) {
            while index > 0 && self.rearranged[index - 1][0] == query[d].1 {
                index -= 1;
            }

            while index < self.rearranged.len() && self.rearranged[index][0] == query[d].1 {
                let mut constrained = true;
                for d2 in 0..DIMENSIONS_MINUS_ONE {
                    constrained &= query[d2 + (d2 >= d) as usize].0 <= self.rearranged[index][d2+1] && self.rearranged[index][d2+1] <= query[d2 + (d2 >= d) as usize].1
                }
                return_val = true;
                self.solved[index] |= (constrained as u8) << 1;
                if self.solved[index] == 0b11 {
                    self.rearranged.remove(index);
                    self.solved.remove(index);
                } else {
                    index += 1;
                }
            }
        }
        return_val
    }
}
