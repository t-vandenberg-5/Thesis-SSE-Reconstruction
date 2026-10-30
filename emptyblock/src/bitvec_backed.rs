use super::*;
use core::{default::Default, iter::Iterator, simd::{Simd, num::SimdUint}};

const SIMD_BLOCK_WIDTH: usize = DOMAIN_USIZE.div_ceil(usize::BITS as usize);

pub struct SimdBlock {
    mask: [[BitArray<[usize; SIMD_BLOCK_WIDTH]>; DOMAIN_USIZE]; DOMAIN_USIZE],
}

impl Default for SimdBlock {
    fn default() -> Self {
        //println!("Called Default!");
        Self { mask: [[BitArray::new([0; SIMD_BLOCK_WIDTH]); DOMAIN_USIZE]; DOMAIN_USIZE] }
    }
}

impl SimdBlock {
    pub fn add_area(&mut self, area: &[(T, T); DIMENSIONS]) {
        //println!("Adding area: {:?}", area);
        for x in area[0].0..=area[0].1 {
            for y in area[1].0..=area[1].1 {
                for z in area[1].0..=area[1].1 {
                    self.mask[x as usize][y as usize].set(z.into(), true);
                }
            }
        }
    }

    pub fn get_pos(&mut self, area: &[(T, T); DIMENSIONS]) -> Option<[T; DIMENSIONS]> {
        //println!("Getting pos: {:?}", area);
        let mut zeroes = 0;
        for x in area[0].0..=area[0].1 {
            for y in area[1].0..=area[1].1 {
                zeroes += self.mask[x as usize][y as usize][area[2].0 as usize..=area[2].1 as usize].count_ones();
                //println!("{x}, {y}: {zeroes}");
                if zeroes > 1 {
                    return None;
                }
            }
        }
        if zeroes == 1 {
            for x in area[0].0..=area[0].1 {
                for y in area[1].0..=area[1].1 {
                    if let Some(z) = self.mask[x as usize][y as usize][area[2].0 as usize..=area[2].1 as usize].first_one() {
                        return Some([x, y, z as T]);
                    }
                }
            }
        }

        None
    }
}
