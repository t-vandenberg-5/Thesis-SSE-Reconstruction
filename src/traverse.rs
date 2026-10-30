type T = u16;

pub trait Traversable<const DIMENSIONS: usize> {
    fn new(dom: T) -> Self;
    fn next(&mut self) -> Option<[T; DIMENSIONS]>;
}

pub struct OnlyMiddle<const DIMENSIONS: usize> {
    pool_start: usize,
    pool_end: usize,
    index: [usize; DIMENSIONS],
    work_arr: Vec<T>,
    has_next: bool,
}

impl<const DIMENSIONS: usize> OnlyMiddle<DIMENSIONS> {
    fn work(dom: T) -> Vec<T> {
        let dom_usize = dom as usize;
        let mut result = Vec::with_capacity(dom_usize);
        let mut stack = Vec::with_capacity(dom_usize + 2);
        stack.push(0);
        stack.push(dom + 1);
        while result.len() < dom_usize {
            let mut i = 1;
            while i < stack.len() {
                let diff = stack[i] - stack[i - 1];
                if diff > 1 {
                    result.push((stack[i - 1] + (diff / 2)) - 1);
                    stack.insert(i, *result.last().unwrap() + 1);
                    i += 1;
                }
                i += 1;
            }
        }
        result
    }
}

impl<const DIMENSIONS: usize> Traversable<DIMENSIONS> for OnlyMiddle<DIMENSIONS> {
    fn new(dom: T) -> Self {
        Self {
            pool_start: 0,
            pool_end: 1,
            index: core::array::from_fn(|_| 0),
            work_arr: OnlyMiddle::<DIMENSIONS>::work(dom),
            has_next: true,
        }
    }

    fn next(&mut self) -> Option<[T; DIMENSIONS]> {
        if self.has_next {
            let pos = core::array::from_fn(|d| self.work_arr[self.index[d]]);
            let mut i = 0;
            // println!("1: {:?}", self.index);
            self.index[i] += 1;
            // println!("2: {:?}", self.index);
            while self.index[i] == self.pool_end && i + 1 < DIMENSIONS {
                self.index[i] = self.pool_start;
                if i + 1 < DIMENSIONS {
                    i += 1;
                    self.index[i] += 1;
                }
                // println!("n: {:?}", self.index);
            }
            // println!("3: {:?}", self.index);

            if self.index[DIMENSIONS - 1] == self.pool_end {
                if self.pool_end == self.work_arr.len() {
                    self.has_next = false;
                    return Some(pos);
                }
                let size = self.pool_end - self.pool_start;
                self.pool_start = self.pool_end;
                self.pool_end = (self.pool_start + (2 * size)).min(self.work_arr.len());
                //println!("{}, {}", self.pool_start, self.pool_end);
                self.index = core::array::from_fn(|_| self.pool_start);
            }
            Some(pos)
        } else {
            None
        }
    }
}

pub struct Full<const DIMENSIONS: usize> {
    pool_start: usize,
    pool_end: usize,
    index: [usize; DIMENSIONS],
    work_arr: Vec<T>,
    has_next: bool,
}

impl<const DIMENSIONS: usize> Traversable<DIMENSIONS> for Full<DIMENSIONS> {
    fn new(dom: T) -> Self {
        Self {
            pool_start: 0,
            pool_end: 1,
            index: core::array::from_fn(|_| 0),
            work_arr: OnlyMiddle::<DIMENSIONS>::work(dom),
            has_next: true,
        }
    }

    fn next(&mut self) -> Option<[T; DIMENSIONS]> {
        if self.has_next {
            let pos = core::array::from_fn(|d| self.work_arr[self.index[d]]);
            let mut i = 0;
            // println!("1: {:?}", self.index);
            self.index[i] += 1;
            // println!("2: {:?}", self.index);
            while self.index[i] == self.pool_end && i + 1 < DIMENSIONS {
                self.index[i] = 0;
                if i + 1 < DIMENSIONS {
                    i += 1;
                    self.index[i] += 1;
                }
                // println!("n: {:?}", self.index);
            }
            // println!("3: {:?}", self.index);

            if self.index[DIMENSIONS - 1] == self.pool_end {
                if self.pool_end == self.work_arr.len() {
                    self.has_next = false;
                    return Some(pos);
                }
                let size = self.pool_end - self.pool_start;
                self.pool_start = self.pool_end;
                self.pool_end = (self.pool_start + (2 * size)).min(self.work_arr.len());
                //println!("{}, {}", self.pool_start, self.pool_end);
                self.index = core::array::from_fn(|d| if d > 0 { 0 } else { self.pool_start });
            }
            Some(pos)
        } else {
            None
        }
    }
}

pub struct Walk<const DIMENSIONS: usize> {
    pos: [T; DIMENSIONS],
    dom: T,
}

impl<const DIMENSIONS: usize> Traversable<DIMENSIONS> for Walk<DIMENSIONS> {
    fn new(dom: T) -> Self {
        Self {
            pos: [0; DIMENSIONS],
            dom,
        }
    }

    fn next(&mut self) -> Option<[T; DIMENSIONS]> {
        if self.pos[DIMENSIONS - 1] != self.dom {
            let v = self.pos;
            self.pos[0] += 1;
            let mut d = 1;
            while d < DIMENSIONS && self.pos[d - 1] == self.dom {
                self.pos[d - 1] = 0;
                self.pos[d] += 1;
                d += 1;
            }
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::traverse::*;

    #[test]
    fn test_only_middle() {
        let mut t = OnlyMiddle::<2>::new(10);
        while let Some(p) = t.next() {
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_walk() {
        let mut t = Walk::<2>::new(10);
        while let Some(p) = t.next() {
            println!("{:?}", p);
        }
    }
}
