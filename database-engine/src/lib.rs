#![feature(portable_simd)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![allow(incomplete_features)]
use std::{cmp::Ordering::{Less, Equal, Greater}, ops::BitOrAssign, simd::Simd};
use nanorand::{BufferedRng, ChaCha8, RandomRange, Rng};

pub const fn qrsw(x: usize) -> usize {
    x.div_ceil(usize::BITS as usize)
}

pub const fn filled_mask<const CAPACITY: usize, const QRMW: usize>() -> QueryResultMask<QRMW> {
    let mut arr = [0; QRMW];
    let mut i = 0;
    while i < QRMW {
        arr[i] = match i.cmp(&(CAPACITY / (usize::BITS as usize))) {
            Less => usize::MAX,
            Equal => {
                if CAPACITY.is_multiple_of(usize::BITS as usize) {
                    usize::MIN
                } else {
                    (1 << (CAPACITY % (usize::BITS as usize))) - 1
                }
            },
            Greater => usize::MIN,
        };

        i += 1;
    }
    Simd::from_array(arr)
}

pub type QueryResultMask<const QRMW: usize> = Simd<usize, QRMW>;

pub struct DB<T, U, const DIMENSIONS: usize, const CAPACITY: usize> {
    data: [[(T, U); CAPACITY]; DIMENSIONS],
}

impl<T: From<u8> + PartialOrd + RandomRange<BufferedRng<ChaCha8, 64>, 64> + Ord + Copy, U: From<u8> + TryFrom<usize> + PartialEq + Copy, const DIMENSIONS: usize, const CAPACITY: usize> DB<T, U, DIMENSIONS, CAPACITY> 
where
    usize: From<U>,
{
    pub fn generate(domain: &[T; DIMENSIONS]) -> Self {
        let mut rand = BufferedRng::new(ChaCha8::new());
        DB {
            data: core::array::from_fn(|d| {
                let mut arr = core::array::from_fn(|r|
                    if r < CAPACITY {
                        (rand.generate_range(T::from(0)..domain[d]), U::try_from(r).or(Err("Could not convert")).unwrap())
                    } else {
                        (T::from(0), U::from(0))
                    }
                );
                let rarr = &mut arr[0..CAPACITY];
                rarr.sort_by_key(|(t, _)| *t);
                arr
            }),
        }
    }

    pub fn read(data: &[Vec<T>; DIMENSIONS]) -> Self {
        DB {
            data: core::array::from_fn(|d| {
                let mut arr = core::array::from_fn(|r|
                    (data[d][r], U::try_from(r).or(Err("Could not convert")).unwrap())
                );
                let rarr = &mut arr[0..CAPACITY];
                rarr.sort_by_key(|(t, _)| *t);
                arr
            }),
        }
    }

    pub fn search<const QRMW: usize>(&self, query: [(T, T); DIMENSIONS]) -> QueryResultMask<QRMW> {
        let mut found = Self::lsr::<QRMW>(&self.data[0], &query[0]);
        (1..DIMENSIONS).for_each(|d| {
            found &= Self::lsr(&self.data[d], &query[d]);
        });
        found
    }

    fn lsr<const QRMW: usize>(data: &[(T, U); CAPACITY], range: &(T, T)) -> QueryResultMask<QRMW> {
        let mut index = 0;
        let mut mask = [usize::MIN; QRMW];
        while index < data.len() && data[index].0 < range.0 {
            index += 1;
        }
        while index < data.len() && data[index].0 <= range.1 {
            <usize as BitOrAssign<usize>>::bitor_assign(&mut mask[usize::from(data[index].1) / (usize::BITS as usize)], 1 << (usize::from(data[index].1) % (usize::BITS as usize)));
            index += 1;
        }
        QueryResultMask::from_array(mask)
    }

    pub fn cheat_pos_traverse(&self, index: U) -> [T; DIMENSIONS] {
        let mut pos = [T::from(0); DIMENSIONS];
        (0..DIMENSIONS).for_each(|d| {
            for (p, id) in self.data[d] {
                if id == index {
                    pos[d] = p;
                    break;
                }
            }
        });
        pos
    }
}
