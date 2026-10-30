#![feature(portable_simd)]
#![feature(const_trait_impl)]
use core::{assert_eq, iter::Iterator, simd::Simd};
use std::{collections::HashMap};

use emptyblock::simd::{SimdBlock3D, SimdBlock2D};
use database_engine::{DB, QueryResultMask, filled_mask};
// use query_engine::BQE;

const fn zero_mask<const QRMW: usize>() -> QueryResultMask<QRMW> {
    Simd::from_array([0; QRMW])
}

const DIMENSIONS2D: usize = 2;
const DIMENSIONS3D: usize = 3;
pub type T = u16;
pub type U = u16;

pub mod midline;
pub mod midline_optimized;
pub mod midline_fast;
pub mod chart;
pub mod traverse;
pub mod dataset;
pub mod util;

pub trait Solver<const DIMENSIONS: usize, const CAPACITY: usize, const QRMW: usize> {
    fn is_solved(&self) -> bool;
    fn get_solved(&self, index: U) -> Option<[T; DIMENSIONS]>;
    fn add_query(&mut self, query: &[(T, T); DIMENSIONS], query_result: QueryResultMask<QRMW>, database: &DB<T, U, DIMENSIONS, CAPACITY>) -> bool;
}

#[derive(Debug)]
pub struct Tracker2D<const DOMAIN_USIZE: usize, const QRMW: usize, const SIMD_BLOCK_WIDTH: usize, const SIMD_LIST_LENGTH: usize> 
{
    currently_solving: QueryResultMask<QRMW>,
    current_area: [(T, T); DIMENSIONS2D],
    pub unsolved_mask: QueryResultMask<QRMW>,
    solved: HashMap<U, [T; DIMENSIONS2D]>,
    sb: Box<SimdBlock2D<T, DOMAIN_USIZE, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>>,
}

impl<const DOMAIN_USIZE: usize, const QRMW: usize, const SIMD_BLOCK_WIDTH: usize, const SIMD_LIST_LENGTH: usize> Tracker2D<DOMAIN_USIZE, QRMW, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH> 
{
    pub fn new<const CAPACITY: usize>(domains: [T; 2]) -> Self
    {
        Self {
            currently_solving: filled_mask::<CAPACITY, QRMW>(),
            current_area: core::array::from_fn(|d| (0, domains[d] - 1)),
            unsolved_mask: filled_mask::<CAPACITY, QRMW>(),
            solved: HashMap::with_capacity(CAPACITY),
            sb: Box::new(SimdBlock2D::default()),
        }
    }

    pub fn reset_current_area(&mut self, domains: [T; 2]) 
    {
        //println!("Reset");
        self.current_area = core::array::from_fn(|d| (0, domains[d] - 1));
        self.currently_solving = self.unsolved_mask;
        *self.sb = SimdBlock2D::default();
    }
}

impl<const DOMAIN_USIZE: usize, const CAPACITY: usize, const QRMW: usize, const SIMD_BLOCK_WIDTH: usize, const SIMD_LIST_LENGTH: usize> Solver<2, CAPACITY, QRMW> for Tracker2D<DOMAIN_USIZE, QRMW, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH> 
{
    fn is_solved(&self) -> bool
    {
        self.unsolved_mask == zero_mask()
    }

    fn get_solved(&self, index: U) -> Option<[T; DIMENSIONS2D]>
    {
        self.solved.get(&index).copied()
    }

    fn add_query(&mut self, query: &[(T, T); DIMENSIONS2D], query_result: QueryResultMask<QRMW>, database: &DB<T, U, DIMENSIONS2D, CAPACITY>) -> bool {
        if query_result & self.currently_solving > zero_mask() {
            // We found a query that pertains to our current solve thing
            let intersect = core::array::from_fn(|d| {
                (
                    query[d].0.max(self.current_area[d].0),
                    query[d].1.min(self.current_area[d].1),
                )
            });
            if intersect != self.current_area {
                let size = intersect.iter().fold(1usize, |prod, i| prod * ((i.1 + 1) - i.0) as usize);
                // We found a smaller area that we get to solve
                let possibly_found_pos = if size == 1 {
                    Some(core::array::from_fn(|d| intersect[d].0))
                } else {
                    self.sb.get_pos(&intersect)
                };
                if let Some(found_pos) = possibly_found_pos {
                    for (index, block) in (query_result & self.currently_solving).to_array().iter().enumerate() {
                        let mut b = *block;
                        while let Some(i) = b.highest_one() {
                            self.solved.insert((i as usize + (index * usize::BITS as usize)) as U, found_pos);
                            assert_eq!(database.cheat_pos_traverse((i as usize + (index * usize::BITS as usize)) as U), found_pos);
                            b -= 1 << i;
                        }
                    }
                    self.unsolved_mask &= !(query_result & self.currently_solving);
                    self.currently_solving &= !(query_result & self.currently_solving);
                    self.sb.add_area(query);
                    if let Some(fp) = self.sb.get_pos(&self.current_area) {
                        for (index, block) in (self.currently_solving).to_array().iter().enumerate() {
                            let mut b = *block;
                            while let Some(i) = b.highest_one() {
                                self.solved.insert((i as usize + (index * usize::BITS as usize)) as U, fp);
                                // println!("Used negative info");
                                assert_eq!(database.cheat_pos_traverse((i as usize + (index * usize::BITS as usize)) as U), fp);
                                b -= 1 << i;
                            }
                        }
                        self.unsolved_mask &= !(self.currently_solving);
                        return true;
                    }
                    return self.currently_solving == zero_mask();
                } else {
                    self.currently_solving &= query_result;
                    self.current_area = intersect;
                }
            }
        } else {
            //println!("Adding area: {:?}, looking for: {:?}, {:?}", query, self.current_area, self.currently_solving);
            self.sb.add_area(query);
            if let Some(p) = self.sb.get_pos(&self.current_area) {
                // println!("{:?}", self.solved);
                for (index, block) in (self.currently_solving).to_array().iter().enumerate() {
                    let mut b = *block;
                    while let Some(i) = b.highest_one() {
                        self.solved.insert((i as usize + (index * usize::BITS as usize)) as U, p);
                        // println!("Used negative info");
                        assert_eq!(database.cheat_pos_traverse((i as usize + (index * usize::BITS as usize)) as U), p);
                        b -= 1 << i;
                    }
                }
                self.unsolved_mask &= !(self.currently_solving);
                return true;
            }
        }
        false
    }
}

pub struct Tracker3D<const DOMAIN_USIZE_0: usize, const DOMAIN_USIZE_1: usize, const QRMW: usize, const SIMD_BLOCK_WIDTH: usize, const SIMD_LIST_LENGTH: usize> 
{
    currently_solving: QueryResultMask<QRMW>,
    current_area: [(T, T); DIMENSIONS3D],
    unsolved_mask: QueryResultMask<QRMW>,
    solved: HashMap<U, [T; DIMENSIONS3D]>,
    sb: Box<SimdBlock3D<T, DOMAIN_USIZE_0, DOMAIN_USIZE_1, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>>,
}

impl<const DOMAIN_USIZE_0: usize, const DOMAIN_USIZE_1: usize, const QRMW: usize, const SIMD_BLOCK_WIDTH: usize, const SIMD_LIST_LENGTH: usize> Tracker3D<DOMAIN_USIZE_0, DOMAIN_USIZE_1, QRMW, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH> 
{
    pub fn new<const CAPACITY: usize>(domains: [T; 3]) -> Self
    {
        Self {
            currently_solving: filled_mask::<CAPACITY, QRMW>(),
            current_area: core::array::from_fn(|d| (0, domains[d] - 1)),
            unsolved_mask: filled_mask::<CAPACITY, QRMW>(),
            solved: HashMap::with_capacity(CAPACITY),
            sb: Box::new(SimdBlock3D::default()),
        }
    }

    pub fn reset_current_area(&mut self, domains: [T; 3]) 
    {
        //println!("Reset");
        self.current_area = core::array::from_fn(|d| (0, domains[d] - 1));
        self.currently_solving = self.unsolved_mask;
        *self.sb = SimdBlock3D::default();
    }
}

impl<const DOMAIN_USIZE_0: usize, const DOMAIN_USIZE_1: usize, const CAPACITY: usize, const QRMW: usize, const SIMD_BLOCK_WIDTH: usize, const SIMD_LIST_LENGTH: usize> Solver<3, CAPACITY, QRMW> for Tracker3D<DOMAIN_USIZE_0, DOMAIN_USIZE_1, QRMW, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH> 
{
    fn is_solved(&self) -> bool 
    {
        self.unsolved_mask == zero_mask()
    }

    fn get_solved(&self, index: U) -> Option<[T; DIMENSIONS3D]> 
    {
        self.solved.get(&index).copied()
    }

    fn add_query(&mut self, query: &[(T, T); DIMENSIONS3D], query_result: QueryResultMask<QRMW>, database: &DB<T, U, DIMENSIONS3D, CAPACITY>) -> bool 
    {
        if query_result & self.currently_solving > zero_mask() {
            // We found a query that pertains to our current solve thing
            let intersect = core::array::from_fn(|d| {
                (
                    query[d].0.max(self.current_area[d].0),
                    query[d].1.min(self.current_area[d].1),
                )
            });
            if intersect != self.current_area {
                let size = intersect.iter().fold(1usize, |prod, i| prod * ((i.1 + 1) - i.0) as usize);
                // We found a smaller area that we get to solve
                let possibly_found_pos = if size == 1 {
                    Some(core::array::from_fn(|d| intersect[d].0))
                } else {
                    self.sb.get_pos(&intersect)
                };
                if let Some(found_pos) = possibly_found_pos {
                    for (index, block) in (query_result & self.currently_solving).to_array().iter().enumerate() {
                        let mut b = *block;
                        while let Some(i) = b.highest_one() {
                            self.solved.insert((i as usize + (index * usize::BITS as usize)) as U, found_pos);
                            assert_eq!(database.cheat_pos_traverse((i as usize + (index * usize::BITS as usize)) as U), found_pos);
                            b -= 1 << i;
                        }
                    }
                    self.unsolved_mask &= !(query_result & self.currently_solving);
                    self.currently_solving &= !(query_result & self.currently_solving);
                    self.sb.add_area(query);
                    if let Some(fp) = self.sb.get_pos(&self.current_area) {
                        for (index, block) in (self.currently_solving).to_array().iter().enumerate() {
                            let mut b = *block;
                            while let Some(i) = b.highest_one() {
                                self.solved.insert((i as usize + (index * usize::BITS as usize)) as U, fp);
                                // println!("Used negative info");
                                assert_eq!(database.cheat_pos_traverse((i as usize + (index * usize::BITS as usize)) as U), fp);
                                b -= 1 << i;
                            }
                        }
                        self.unsolved_mask &= !(self.currently_solving);
                        return true;
                    }
                    return self.currently_solving == zero_mask();
                } else {
                    self.currently_solving &= query_result;
                    self.current_area = intersect;
                }
            }
        } else {
            //println!("Adding area: {:?}, looking for: {:?}, {:?}", query, self.current_area, self.currently_solving);
            self.sb.add_area(query);
            if let Some(p) = self.sb.get_pos(&self.current_area) {
                // println!("{:?}", self.solved);
                for (index, block) in (self.currently_solving).to_array().iter().enumerate() {
                    let mut b = *block;
                    while let Some(i) = b.highest_one() {
                        self.solved.insert((i as usize + (index * usize::BITS as usize)) as U, p);
                        // println!("Used negative info");
                        assert_eq!(database.cheat_pos_traverse((i as usize + (index * usize::BITS as usize)) as U), p);
                        b -= 1 << i;
                    }
                }
                self.unsolved_mask &= !(self.currently_solving);
                return true;
            }
        }
        false
    }
}
