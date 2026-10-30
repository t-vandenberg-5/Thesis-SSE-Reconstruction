

pub struct SimdBlock {}

impl Default for SimdBlock {
    fn default() -> Self {
    }
}

impl SimdBlock {
    pub fn add_area(&mut self, area: &[(T, T); DIMENSIONS]) {
    }

    pub fn get_pos(&mut self, area: &[(T, T); DIMENSIONS]) -> Option<[T; DIMENSIONS]> {
    }
}

#[cfg(test)]
mod tests {
    use core::assert_eq;

    use super::*;

    #[test]
    fn test_blocklen() {
        assert_eq!(blocklen((0, 0)), Simd::from_array(core::array::from_fn(|b| {
            if b > 0 {
                usize::MIN
            } else {
                0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            }
        })));
        assert_eq!(blocklen((0, 5)), Simd::from_array(core::array::from_fn(|b| {
            if b > 0 {
                usize::MIN
            } else {
                0b1111_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            }
        })));
        assert_eq!(blocklen((3, 7)), Simd::from_array(core::array::from_fn(|b| {
            if b > 0 {
                usize::MIN
            } else {
                0b0001_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            }
        })));
        assert_eq!(blocklen((3, 5)), Simd::from_array(core::array::from_fn(|b| {
            if b > 0 {
                usize::MIN
            } else {
                0b0001_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            }
        })));
        assert_eq!(blocklen((0, 64)), Simd::from_array(core::array::from_fn(|b| {
            if b > 0 {
                0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            } else {
                usize::MAX
            }
        })));
        assert_eq!(blocklen((1, 65)), Simd::from_array(core::array::from_fn(|b| {
            if b > 0 {
                0b1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            } else {
                0b0111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111
            }
        })));
        assert_eq!(blocklen((64, 83)), Simd::from_array(core::array::from_fn(|b| {
            if b > 0 {
                0b1111_1111_1111_1111_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            } else {
                0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000
            }
        })));
    }
}
