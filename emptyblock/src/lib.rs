#![feature(portable_simd)]
#![feature(step_trait)]
// use bitvec::prelude::BitArray;

// pub type MaskBlock = u8;
// pub const MASK_BLOCK_COUNT: usize = DOMAIN_SIZE.div_ceil(MaskBlock::BITS as usize);
// pub type Mask = BitArray<[MaskBlock; MASK_BLOCK_COUNT], bitvec::order::Msb0>;

//pub mod naive;
pub mod simd;
//pub mod bitvec_backed;

pub trait EmptyBlock<T, const DIMENSIONS: usize> {
    fn add_empty_area(&mut self, query: [(T, T); DIMENSIONS]);
    fn is_single_pos(&self) -> Option<[T; DIMENSIONS]>;
}

//pub type DefaultEmptyBlock = naive::Naive;
