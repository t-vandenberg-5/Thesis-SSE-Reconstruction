use core::{
    default::Default,
    iter::Iterator,
    simd::{Simd, num::SimdUint},
};
use std::{
    iter::Step,
    marker::PhantomData,
    ops::{Add, Sub, SubAssign},
};

const MAX_SIMD_LEN: u8 = 8;
const MAX_SIMD_LEN_USIZE: usize = MAX_SIMD_LEN as usize;
const TOTAL_BLOCK_SIZE: u16 = MAX_SIMD_LEN as u16 * usize::BITS as u16;

const DIMENSIONS2D: usize = 2;
const DIMENSIONS3D: usize = 3;

pub const fn simd_block_width(n: usize) -> (usize, usize) {
    let ml = n.div_ceil(usize::BITS as usize);
    if ml > MAX_SIMD_LEN_USIZE {
        (MAX_SIMD_LEN_USIZE, ml.div_ceil(MAX_SIMD_LEN_USIZE))
    } else {
        (ml, 1)
    }
}

fn blocklen<
    T: TryFrom<u16> + SubAssign + PartialOrd + Add<Output = T> + From<u8> + Sub<Output = T> + Copy,
    const SIMD_BLOCK_WIDTH: usize,
    const SIMD_LIST_LENGTH: usize,
>(
    mut block: (T, T),
) -> (
    (Simd<usize, SIMD_BLOCK_WIDTH>, usize),
    Option<(Simd<usize, SIMD_BLOCK_WIDTH>, usize)>,
)
where
    usize: From<T>,
{
    let mut pos = 0;

    if let Ok(tbs) = T::try_from(TOTAL_BLOCK_SIZE) {
        while block.0 >= tbs {
            block.0 -= tbs;
            block.1 -= tbs;
            pos += 1;
        }
    }
    let mut start = block.0.into();
    let mut blocklen: usize = ((block.1 + T::from(1)) - block.0).into();
    let start_res = if start >= usize::BITS as usize * MAX_SIMD_LEN_USIZE {
        Simd::from_array(core::array::from_fn(|_| 0))
    } else {
        Simd::from_array(core::array::from_fn(|_| {
            if start >= usize::BITS as usize {
                start -= usize::BITS as usize;
                usize::MIN
            } else if start > 0 {
                let r = 1usize
                    .unbounded_shl(blocklen.min(usize::BITS as usize) as u32)
                    .wrapping_sub(1usize)
                    .reverse_bits()
                    >> start;
                blocklen -= blocklen.min(usize::BITS as usize - start);
                start = 0;
                r
            } else {
                if blocklen >= usize::BITS as usize {
                    blocklen -= usize::BITS as usize;
                    usize::MAX
                } else if blocklen == 0 {
                    usize::MIN
                } else {
                    let old = blocklen;
                    blocklen = 0;
                    ((1usize.unbounded_shl(old as u32)) - 1).reverse_bits()
                }
            }
        }))
    };
    (
        (start_res, pos),
        if let Ok(tbs) = T::try_from(TOTAL_BLOCK_SIZE)
            && block.1 >= tbs
        {
            let mut pos2 = pos;
            while block.1 >= tbs {
                if block.0 >= tbs {
                    block.0 -= tbs;
                } else {
                    block.0 = T::from(0);
                }
                block.1 -= tbs;
                pos2 += 1;
            }
            start = block.0.into();
            blocklen = ((block.1 + T::from(1)) - block.0).into();
            Some((
                if start >= usize::BITS as usize * MAX_SIMD_LEN_USIZE {
                    Simd::from_array(core::array::from_fn(|_| 0))
                } else {
                    Simd::from_array(core::array::from_fn(|_| {
                        if start >= usize::BITS as usize {
                            start -= usize::BITS as usize;
                            usize::MIN
                        } else if start > 0 {
                            let r = 1usize
                                .unbounded_shl(blocklen.min(64) as u32)
                                .wrapping_sub(1usize)
                                .reverse_bits()
                                >> start;
                            blocklen -= blocklen.min(usize::BITS as usize - start);
                            start = 0;
                            r
                        } else {
                            if blocklen >= usize::BITS as usize {
                                blocklen -= usize::BITS as usize;
                                usize::MAX
                            } else if blocklen == 0 {
                                usize::MIN
                            } else {
                                let old = blocklen;
                                blocklen = 0;
                                ((1usize.unbounded_shl(old as u32)) - 1).reverse_bits()
                            }
                        }
                    }))
                },
                pos2,
            ))
        } else {
            None
        },
    )
}

#[derive(Debug)]
pub struct SimdBlock2D<
    T,
    const DOMAIN_USIZE: usize,
    const SIMD_BLOCK_WIDTH: usize,
    const SIMD_LIST_LENGTH: usize,
> {
    t: std::marker::PhantomData<T>,
    mask: Box<[[Simd<usize, SIMD_BLOCK_WIDTH>; SIMD_LIST_LENGTH]; DOMAIN_USIZE]>,
}

impl<T, const DOMAIN_USIZE: usize, const SIMD_BLOCK_WIDTH: usize, const SIMD_LIST_LENGTH: usize>
    Default for SimdBlock2D<T, DOMAIN_USIZE, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>
{
    fn default() -> Self {
        //println!("Called Default!");
        Self {
            t: PhantomData::<T>,
            mask: vec![[Simd::from_array([0; SIMD_BLOCK_WIDTH]); SIMD_LIST_LENGTH]; DOMAIN_USIZE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        }
    }
}

impl<
    T: TryFrom<u16>
        + TryFrom<usize>
        + SubAssign
        + PartialOrd
        + Add<Output = T>
        + From<u8>
        + Sub<Output = T>
        + Copy
        + Step,
    const DOMAIN_USIZE: usize,
    const SIMD_BLOCK_WIDTH: usize,
    const SIMD_LIST_LENGTH: usize,
> SimdBlock2D<T, DOMAIN_USIZE, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>
where
    usize: From<T>,
{
    pub fn add_area(&mut self, area: &[(T, T); DIMENSIONS2D]) {
        //println!("Adding area: {:?}", area);
        let blocklen = blocklen::<T, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>(area[DIMENSIONS2D - 1]);
        if let ((start_block, start_pos), Some((end_block, end_pos))) = blocklen {
            for x in area[0].0..=area[0].1 {
                self.mask[usize::from(x)][start_pos] |= start_block;
                for y in start_pos + 1..end_pos {
                    self.mask[usize::from(x)][y] |=
                        Simd::from_array([usize::MAX; SIMD_BLOCK_WIDTH]);
                }
                self.mask[usize::from(x)][end_pos] |= end_block;
            }
        } else {
            for x in area[0].0..=area[0].1 {
                self.mask[usize::from(x)][blocklen.0.1] |= blocklen.0.0;
            }
        }
    }

    pub fn get_pos(&mut self, area: &[(T, T); DIMENSIONS2D]) -> Option<[T; DIMENSIONS2D]> {
        //println!("Getting pos: {:?}", area);
        let mut zeroes = 0;
        let blocklen = blocklen::<T, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>(area[DIMENSIONS2D - 1]);
        if let ((start_block, start_pos), Some((end_block, end_pos))) = blocklen {
            for x in area[0].0..=area[0].1 {
                zeroes += (self.mask[usize::from(x)][start_pos] | !start_block)
                    .count_zeros()
                    .to_array()
                    .iter()
                    .sum::<usize>();
                for y in start_pos + 1..end_pos {
                    zeroes += self.mask[usize::from(x)][y]
                        .count_zeros()
                        .to_array()
                        .iter()
                        .sum::<usize>();
                }
                zeroes += (self.mask[usize::from(x)][end_pos] | !end_block)
                    .count_zeros()
                    .to_array()
                    .iter()
                    .sum::<usize>();
                //println!("{x}, {y}: {zeroes}");
                if zeroes > 1 {
                    return None;
                }
            }
        } else {
            for x in area[0].0..=area[0].1 {
                zeroes += (self.mask[usize::from(x)][blocklen.0.1] | !blocklen.0.0)
                    .count_zeros()
                    .to_array()
                    .iter()
                    .sum::<usize>();
                //println!("{x}, {y}: {zeroes}");
                if zeroes > 1 {
                    return None;
                }
            }
        }
        if zeroes == 1 {
            for x in area[0].0..=area[0].1 {
                match blocklen {
                    ((start_block, start_pos), Some((end_block, end_pos))) => {
                        if let Some((block, index)) = (self.mask[usize::from(x)][start_pos]
                            | !start_block)
                            .leading_ones()
                            .to_array()
                            .iter()
                            .enumerate()
                            .find(|(_, x)| **x != usize::BITS as usize)
                        {
                            return Some([
                                x,
                                T::try_from(
                                    ((block + (start_pos * MAX_SIMD_LEN_USIZE))
                                        * usize::BITS as usize)
                                        + index,
                                )
                                .or(Err("Could not convert"))
                                .unwrap(),
                            ]);
                        };
                        for y in start_pos + 1..end_pos {
                            if let Some((block, index)) = (self.mask[usize::from(x)][y])
                                .leading_ones()
                                .to_array()
                                .iter()
                                .enumerate()
                                .find(|(_, x)| **x != usize::BITS as usize)
                            {
                                return Some([
                                    x,
                                    T::try_from(
                                        ((block + (y * MAX_SIMD_LEN_USIZE)) * usize::BITS as usize)
                                            + index,
                                    )
                                    .or(Err("Could not convert"))
                                    .unwrap(),
                                ]);
                            };
                        }
                        if let Some((block, index)) = (self.mask[usize::from(x)][end_pos]
                            | !end_block)
                            .leading_ones()
                            .to_array()
                            .iter()
                            .enumerate()
                            .find(|(_, x)| **x != usize::BITS as usize)
                        {
                            return Some([
                                x,
                                T::try_from(
                                    ((block + (end_pos * MAX_SIMD_LEN_USIZE))
                                        * usize::BITS as usize)
                                        + index,
                                )
                                .or(Err("Could not convert"))
                                .unwrap(),
                            ]);
                        };
                    }
                    ((start_block, start_pos), None) => {
                        if let Some((block, index)) = (self.mask[usize::from(x)][start_pos]
                            | !start_block)
                            .leading_ones()
                            .to_array()
                            .iter()
                            .enumerate()
                            .find(|(_, x)| **x != usize::BITS as usize)
                        {
                            return Some([
                                x,
                                T::try_from(
                                    ((block + (start_pos * MAX_SIMD_LEN_USIZE))
                                        * usize::BITS as usize)
                                        + index,
                                )
                                .or(Err("Could not convert"))
                                .unwrap(),
                            ]);
                        };
                    }
                }
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct SimdBlock3D<
    T,
    const DOMAIN_USIZE_0: usize,
    const DOMAIN_USIZE_1: usize,
    const SIMD_BLOCK_WIDTH: usize,
    const SIMD_LIST_LENGTH: usize,
> {
    t: std::marker::PhantomData<T>,
    mask: Box<
        [Box<[[Simd<usize, SIMD_BLOCK_WIDTH>; SIMD_LIST_LENGTH]; DOMAIN_USIZE_0]>; DOMAIN_USIZE_1],
    >,
}

impl<
    T,
    const DOMAIN_USIZE_0: usize,
    const DOMAIN_USIZE_1: usize,
    const SIMD_BLOCK_WIDTH: usize,
    const SIMD_LIST_LENGTH: usize,
> Default for SimdBlock3D<T, DOMAIN_USIZE_0, DOMAIN_USIZE_1, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>
{
    fn default() -> Self {
        //println!("Called Default!");
        Self {
            t: PhantomData::<T>,
            mask: vec![
                vec![
                    [Simd::from_array([0; SIMD_BLOCK_WIDTH]); SIMD_LIST_LENGTH];
                    DOMAIN_USIZE_0
                ]
                .into_boxed_slice()
                .try_into()
                .unwrap();
                DOMAIN_USIZE_1
            ]
            .into_boxed_slice()
            .try_into()
            .unwrap(),
        }
    }
}

impl<
    T: TryFrom<u16>
        + TryFrom<usize>
        + SubAssign
        + PartialOrd
        + Add<Output = T>
        + From<u8>
        + Sub<Output = T>
        + Copy
        + Step,
    const DOMAIN_USIZE_0: usize,
    const DOMAIN_USIZE_1: usize,
    const SIMD_BLOCK_WIDTH: usize,
    const SIMD_LIST_LENGTH: usize,
> SimdBlock3D<T, DOMAIN_USIZE_0, DOMAIN_USIZE_1, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>
where
    usize: From<T>,
{
    pub fn add_area(&mut self, area: &[(T, T); DIMENSIONS3D]) {
        //println!("Adding area: {:?}", area);
        let blocklen = blocklen::<T, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>(area[DIMENSIONS3D - 1]);
        if let ((start_block, start_pos), Some((end_block, end_pos))) = blocklen {
            for x in area[0].0..=area[0].1 {
                for y in area[1].0..=area[1].1 {
                    self.mask[usize::from(x)][usize::from(y)][start_pos] |= start_block;
                    for z in start_pos + 1..end_pos {
                        self.mask[usize::from(x)][usize::from(y)][z] |=
                            Simd::from_array([usize::MAX; SIMD_BLOCK_WIDTH]);
                    }
                    self.mask[usize::from(x)][usize::from(y)][end_pos] |= end_block;
                }
            }
        } else {
            for x in area[0].0..=area[0].1 {
                for y in area[1].0..=area[1].1 {
                    self.mask[usize::from(x)][usize::from(y)][blocklen.0.1] |= blocklen.0.0;
                }
            }
        }
    }

    pub fn get_pos(&mut self, area: &[(T, T); DIMENSIONS3D]) -> Option<[T; DIMENSIONS3D]> {
        //println!("Getting pos: {:?}", area);
        let mut zeroes = 0;
        let blocklen = blocklen::<T, SIMD_BLOCK_WIDTH, SIMD_LIST_LENGTH>(area[DIMENSIONS3D - 1]);
        if let ((start_block, start_pos), Some((end_block, end_pos))) = blocklen {
            for x in area[0].0..=area[0].1 {
                for y in area[1].0..=area[1].1 {
                    zeroes += (self.mask[usize::from(x)][usize::from(y)][start_pos] | !start_block)
                        .count_zeros()
                        .to_array()
                        .iter()
                        .sum::<usize>();
                    for z in start_pos + 1..end_pos {
                        zeroes += self.mask[usize::from(x)][usize::from(y)][z]
                            .count_zeros()
                            .to_array()
                            .iter()
                            .sum::<usize>();
                    }
                    zeroes += (self.mask[usize::from(x)][usize::from(y)][end_pos] | !end_block)
                        .count_zeros()
                        .to_array()
                        .iter()
                        .sum::<usize>();
                    //println!("{x}, {y}: {zeroes}");
                    if zeroes > 1 {
                        return None;
                    }
                }
            }
        } else {
            for x in area[0].0..=area[0].1 {
                for y in area[1].0..=area[1].1 {
                    zeroes += (self.mask[usize::from(x)][usize::from(y)][blocklen.0.1]
                        | !blocklen.0.0)
                        .count_zeros()
                        .to_array()
                        .iter()
                        .sum::<usize>();
                    //println!("{x}, {y}: {zeroes}");
                    if zeroes > 1 {
                        return None;
                    }
                }
            }
        }
        if zeroes == 1 {
            for x in area[0].0..=area[0].1 {
                for y in area[1].0..=area[1].1 {
                    match blocklen {
                        ((start_block, start_pos), Some((end_block, end_pos))) => {
                            if let Some((block, index)) = (self.mask[usize::from(x)]
                                [usize::from(y)][start_pos]
                                | !start_block)
                                .leading_ones()
                                .to_array()
                                .iter()
                                .enumerate()
                                .find(|(_, x)| **x != usize::BITS as usize)
                            {
                                return Some([
                                    x,
                                    y,
                                    T::try_from(
                                        ((block + (start_pos * MAX_SIMD_LEN_USIZE))
                                            * usize::BITS as usize)
                                            + index,
                                    )
                                    .or(Err("Could not convert"))
                                    .unwrap(),
                                ]);
                            };
                            for z in start_pos + 1..end_pos {
                                if let Some((block, index)) = (self.mask[usize::from(x)]
                                    [usize::from(y)][z])
                                    .leading_ones()
                                    .to_array()
                                    .iter()
                                    .enumerate()
                                    .find(|(_, x)| **x != usize::BITS as usize)
                                {
                                    return Some([
                                        x,
                                        y,
                                        T::try_from(
                                            ((block + (z * MAX_SIMD_LEN_USIZE))
                                                * usize::BITS as usize)
                                                + index,
                                        )
                                        .or(Err("Could not convert"))
                                        .unwrap(),
                                    ]);
                                };
                            }
                            if let Some((block, index)) =
                                (self.mask[usize::from(x)][usize::from(y)][end_pos] | !end_block)
                                    .leading_ones()
                                    .to_array()
                                    .iter()
                                    .enumerate()
                                    .find(|(_, x)| **x != usize::BITS as usize)
                            {
                                return Some([
                                    x,
                                    y,
                                    T::try_from(
                                        ((block + (end_pos * MAX_SIMD_LEN_USIZE))
                                            * usize::BITS as usize)
                                            + index,
                                    )
                                    .or(Err("Could not convert"))
                                    .unwrap(),
                                ]);
                            };
                        }
                        ((start_block, start_pos), None) => {
                            if let Some((block, index)) = (self.mask[usize::from(x)]
                                [usize::from(y)][start_pos]
                                | !start_block)
                                .leading_ones()
                                .to_array()
                                .iter()
                                .enumerate()
                                .find(|(_, x)| **x != usize::BITS as usize)
                            {
                                return Some([
                                    x,
                                    y,
                                    T::try_from(
                                        ((block + (start_pos * MAX_SIMD_LEN_USIZE))
                                            * usize::BITS as usize)
                                            + index,
                                    )
                                    .or(Err("Could not convert"))
                                    .unwrap(),
                                ]);
                            };
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::simd::Simd;

    type T = u16;

    use crate::simd::{SimdBlock2D, SimdBlock3D, blocklen, simd_block_width};
    #[test]
    fn test_simd_block_width() {
        assert_eq!(simd_block_width(0), (0, 1));
        assert_eq!(simd_block_width(1), (1, 1));
        assert_eq!(simd_block_width(63), (1, 1));
        assert_eq!(simd_block_width(64), (1, 1));
        assert_eq!(simd_block_width(65), (2, 1));
        assert_eq!(simd_block_width(127), (2, 1));
        assert_eq!(simd_block_width(128), (2, 1));
        assert_eq!(simd_block_width(129), (3, 1));
        assert_eq!(simd_block_width(191), (3, 1));
        assert_eq!(simd_block_width(192), (3, 1));
        assert_eq!(simd_block_width(193), (4, 1));
        assert_eq!(simd_block_width(255), (4, 1));
        assert_eq!(simd_block_width(256), (4, 1));
        assert_eq!(simd_block_width(257), (5, 1));
        assert_eq!(simd_block_width(319), (5, 1));
        assert_eq!(simd_block_width(320), (5, 1));
        assert_eq!(simd_block_width(321), (6, 1));
        assert_eq!(simd_block_width(383), (6, 1));
        assert_eq!(simd_block_width(384), (6, 1));
        assert_eq!(simd_block_width(385), (7, 1));
        assert_eq!(simd_block_width(447), (7, 1));
        assert_eq!(simd_block_width(448), (7, 1));
        assert_eq!(simd_block_width(449), (8, 1));
        assert_eq!(simd_block_width(511), (8, 1));
        assert_eq!(simd_block_width(512), (8, 1));
        assert_eq!(simd_block_width(513), (8, 2));
        assert_eq!(simd_block_width(1023), (8, 2));
        assert_eq!(simd_block_width(1024), (8, 2));
        assert_eq!(simd_block_width(1025), (8, 3));
    }

    #[test]
    fn test_bounds() {
        const SBW: usize = simd_block_width(4096).0;
        const SLL: usize = simd_block_width(4096).1;

        let ((_, start_pos), other) =
            blocklen::<u16, { simd_block_width(128).0 }, { simd_block_width(128).1 }>((0, 128));
        assert_eq!(start_pos, 0);
        assert_eq!(other, None);
        let ((_, start_pos), other) =
            blocklen::<u16, { simd_block_width(512).0 }, { simd_block_width(512).1 }>((0, 511));
        assert_eq!(start_pos, 0);
        assert_eq!(other, None);
        let ((start_block, start_pos), other) = blocklen::<u16, SBW, SLL>((0, 511));
        assert_eq!(start_block, Simd::from_array([usize::MAX; SBW]));
        assert_eq!(start_pos, 0);
        assert_eq!(other, None);

        let ((_, start_pos), other) = blocklen::<u16, SBW, SLL>((512, 512));
        assert_eq!(start_pos, 1);
        assert_eq!(other, None);

        let ((start_block, start_pos), other) = blocklen::<u16, SBW, SLL>((0, 0));
        assert_eq!(
            start_block,
            Simd::from_array(core::array::from_fn(|p| if p > 0 { 0 } else { 1 << 63 }))
        );
        assert_eq!(start_pos, 0);
        assert_eq!(other, None);

        let ((start_block, start_pos), other) = blocklen::<u16, SBW, SLL>((0, 513));
        assert_eq!(start_pos, 0);
        assert_eq!(start_block, Simd::from_array([usize::MAX; SBW]));
        if let Some((end_block, end_pos)) = other {
            assert_eq!(end_pos, 1);
            assert_eq!(
                end_block,
                Simd::from_array(core::array::from_fn(|p| if p > 0 { 0 } else { 3 << 62 }))
            );
        } else {
            panic!("Should have other")
        }
    }

    #[test]
    fn big() {
        const DOMAIN_USIZE: usize = 4096;
        const SBW: usize = simd_block_width(DOMAIN_USIZE).0;
        const SLL: usize = simd_block_width(DOMAIN_USIZE).1;
        let mut eb = SimdBlock2D::<u16, DOMAIN_USIZE, SBW, SLL>::default();
        eb.add_area(&[(0 as T, 0 as T), (0 as T, 0 as T)]);
        assert_eq!(
            eb.get_pos(&[(0 as T, 0 as T), (0 as T, 1 as T)]),
            Some([0, 1])
        );
    }

    #[test]
    fn medium3d() {
        const DOMAIN_USIZE: usize = 100;
        const SBW: usize = simd_block_width(DOMAIN_USIZE).0;
        const SLL: usize = simd_block_width(DOMAIN_USIZE).1;
        let mut eb = SimdBlock3D::<u16, DOMAIN_USIZE, DOMAIN_USIZE, SBW, SLL>::default();
        eb.add_area(&[(0 as T, 0 as T), (0 as T, 0 as T), (0 as T, 0 as T)]);
        assert_eq!(
            eb.get_pos(&[(0 as T, 0 as T), (0 as T, 1 as T), (0 as T, 0 as T)]),
            Some([0, 1, 0])
        );
    }
}
