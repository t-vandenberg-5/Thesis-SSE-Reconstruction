use super::*;

pub struct Naive {
    mask: Mask,
}

impl Default for Naive {
    fn default() -> Self {
        Self { mask: Mask::new([0; MASK_BLOCK_COUNT]) }
    }
}

impl EmptyBlock for Naive {
    fn add_empty_area(&mut self, query: [(T, T); DIMENSIONS]) {
        for x in query[0].0..=query[0].1 {
            for y in query[1].0..=query[1].1 {
                for z in query[2].0..=query[2].1 {
                    self.mask.set(x as usize + ((y as usize + (z as usize * DOMAIN_USIZE)) * DOMAIN_USIZE), true);
                }
            }
        }
    }

    fn is_single_pos(&self) -> Option<[T; DIMENSIONS]>{
        match self.mask.count_ones() {
            val if val == DOMAIN_SIZE - 1 => {
                let mut encoded_pos = self.mask.first_zero().unwrap();
                println!("op{}", encoded_pos);
                let mut pos = [0; DIMENSIONS];
                for d in 0..DIMENSIONS {
                    pos[d] = (encoded_pos % DOMAIN_USIZE) as T;
                    encoded_pos /= DOMAIN_USIZE;
                }
                Some(pos)
            }
            _ => None
        }
    }
}




#[cfg(test)]
mod tests {
    use std::array::from_fn;

    use nanorand::{BufferedRng, ChaCha8, Rng};

    use super::*;

    fn test_empty_block_random() {
        let mut rand = BufferedRng::new(ChaCha8::new());
        for _ in 0..1 {
            let pos: [T; DIMENSIONS] = from_fn(|_| rand.generate_range(0..DOMAIN));
            println!("POS: {:?}", pos);
            let mut eb = Naive::default();
            while eb.is_single_pos().is_none() {
                let query = from_fn(|_| {
                    let a = rand.generate_range(0..DOMAIN);
                    let b = rand.generate_range(0..DOMAIN);
                    if a < b {
                        (a, b)
                    } else {
                        (b, a)
                    }
                });
                let mut intersects = true;
                for d in 0..DIMENSIONS {
                    if query[d].0 > pos[d] || query[d].1 < pos[d] {
                        intersects = false;
                    }
                }
                if !intersects {
                    eb.add_empty_area(query);
                }
            }
            assert_eq!(eb.is_single_pos().unwrap(), pos);
        }

    }
}
