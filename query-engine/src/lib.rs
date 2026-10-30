pub mod generator {
    use nanorand::{BufferedRng, RandomRange, Rng};

    pub trait GeneratorTrait<T, const DIMENSIONS: usize>: Default {
        type Generated: Copy;

        fn generate(&mut self, domain: &[T; DIMENSIONS]) -> Self::Generated;
    }

    #[derive(Clone)]
    pub struct RandomGenerator<
        RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH>,
        const RNG_SOURCE_WIDTH: usize,
    > {
        rand: BufferedRng<RNGSource, RNG_SOURCE_WIDTH>,
    }

    impl<RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH> + Default, const RNG_SOURCE_WIDTH: usize>
        Default for RandomGenerator<RNGSource, RNG_SOURCE_WIDTH>
    {
        fn default() -> Self {
            Self {
                rand: BufferedRng::new(RNGSource::default()),
            }
        }
    }

    impl<
        T: From<u8> + PartialOrd + Copy,
        const DIMENSIONS: usize,
        RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH> + Default,
        const RNG_SOURCE_WIDTH: usize,
    > GeneratorTrait<T, DIMENSIONS> for RandomGenerator<RNGSource, RNG_SOURCE_WIDTH>
    where
        T: RandomRange<BufferedRng<RNGSource, RNG_SOURCE_WIDTH>, RNG_SOURCE_WIDTH>,
    {
        type Generated = [(T, T); DIMENSIONS];

        fn generate(&mut self, domain: &[T; DIMENSIONS]) -> Self::Generated {
            core::array::from_fn(|d| {
                let a = self.rand.generate_range(T::from(0)..domain[d]);
                let b = self.rand.generate_range(T::from(0)..domain[d]);
                if a <= b { (a, b) } else { (b, a) }
            })
        }
    }

    #[derive(Clone)]
    pub struct UniformRandomGenerator<
        RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH>,
        const RNG_SOURCE_WIDTH: usize,
    > {
        rand: BufferedRng<RNGSource, RNG_SOURCE_WIDTH>,
    }

    impl<RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH> + Default, const RNG_SOURCE_WIDTH: usize>
        Default for UniformRandomGenerator<RNGSource, RNG_SOURCE_WIDTH>
    {
        fn default() -> Self {
            Self {
                rand: BufferedRng::new(RNGSource::default()),
            }
        }
    }

    impl<
        T: From<u8> + PartialOrd + Copy,
        const DIMENSIONS: usize,
        RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH> + Default,
        const RNG_SOURCE_WIDTH: usize,
    > GeneratorTrait<T, DIMENSIONS> for UniformRandomGenerator<RNGSource, RNG_SOURCE_WIDTH>
    where
        T: RandomRange<BufferedRng<RNGSource, RNG_SOURCE_WIDTH>, RNG_SOURCE_WIDTH>,
    {
        type Generated = [(T, T); DIMENSIONS];

        fn generate(&mut self, domain: &[T; DIMENSIONS]) -> Self::Generated {
            core::array::from_fn(|d| {
                let a = self.rand.generate_range(T::from(0)..=domain[d]);
                let b = self.rand.generate_range(T::from(0)..domain[d]);
                if a == domain[d] {
                    (b, b)
                } else if a < b {
                    (a, b)
                } else {
                    (b, a)
                }
            })
        }
    }

    #[derive(Clone)]
    pub struct BetterRandomGenerator<
        RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH>,
        const RNG_SOURCE_WIDTH: usize,
    > {
        rand: BufferedRng<RNGSource, RNG_SOURCE_WIDTH>,
    }

    impl<RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH> + Default, const RNG_SOURCE_WIDTH: usize>
        Default for BetterRandomGenerator<RNGSource, RNG_SOURCE_WIDTH>
    {
        fn default() -> Self {
            Self {
                rand: BufferedRng::new(RNGSource::default()),
            }
        }
    }

    impl<
        T: From<u8>
            + RandomRange<BufferedRng<RNGSource, RNG_SOURCE_WIDTH>, RNG_SOURCE_WIDTH>
            + PartialOrd
            + Copy,
        const DIMENSIONS: usize,
        RNGSource: nanorand::Rng<RNG_SOURCE_WIDTH> + Default,
        const RNG_SOURCE_WIDTH: usize,
    > GeneratorTrait<T, DIMENSIONS> for BetterRandomGenerator<RNGSource, RNG_SOURCE_WIDTH>
    {
        type Generated = [(T, T); DIMENSIONS];

        fn generate(&mut self, domain: &[T; DIMENSIONS]) -> Self::Generated {
            core::array::from_fn(|d| {
                let a = self.rand.generate_range(T::from(0)..domain[d]);
                let b = self.rand.generate_range(a..domain[d]);
                (a, b)
            })
        }
    }
}

pub mod buffer {
    use bumpalo::{Bump, collections::Vec};

    use crate::generator::GeneratorTrait;
    pub trait BufferTrait<T, const DIMENSIONS: usize> {
        type Generated;
        fn next(&mut self, domain: &[T; DIMENSIONS]) -> Self::Generated;
        fn existing_next(&mut self) -> Option<Self::Generated>;
        fn restart(&mut self);
        fn size(&self) -> usize;
    }

    pub struct Buffer<'a, T, const DIMENSIONS: usize, Generator: GeneratorTrait<T, DIMENSIONS>> {
        generator: Generator,
        index: usize,
        vec: Vec<'a, Generator::Generated>,
    }

    impl<'a, T, const DIMENSIONS: usize, Generator: GeneratorTrait<T, DIMENSIONS>>
        Buffer<'a, T, DIMENSIONS, Generator>
    {
        pub fn new(bump: &'a Bump) -> Self {
            Self {
                generator: Generator::default(),
                index: 0,
                vec: Vec::new_in(bump),
            }
        }
    }

    impl<'a, T, const DIMENSIONS: usize, Generator: GeneratorTrait<T, DIMENSIONS>>
        BufferTrait<T, DIMENSIONS> for Buffer<'a, T, DIMENSIONS, Generator>
    {
        type Generated = Generator::Generated;
        fn existing_next(&mut self) -> Option<Generator::Generated> {
            if self.index == self.vec.len() {
                None
            } else {
                self.index += 1;
                Some(self.vec[self.index - 1])
            }
        }
        fn next(&mut self, domain: &[T; DIMENSIONS]) -> Generator::Generated {
            if self.index == self.vec.len() {
                self.vec.push(self.generator.generate(domain));
            }
            self.index += 1;
            self.vec[self.index - 1]
        }

        fn restart(&mut self) {
            self.index = 0;
        }

        fn size(&self) -> usize {
            self.vec.len()
        }
    }

    pub struct LoadedBuffer<'a, T> {
        index: usize,
        vec: Vec<'a, T>,
    }

    impl<'a, T> LoadedBuffer<'a, T> {
        pub fn new(bump: &'a Bump) -> Self {
            Self {
                index: 0,
                vec: Vec::new_in(bump),
            }
        }

        pub fn load(&mut self, value: T) {
            self.vec.push(value);
        }
    }

    impl<'a, T, const DIMENSIONS: usize, X: Copy> BufferTrait<T, DIMENSIONS> for LoadedBuffer<'a, X> {
        type Generated = X;
        fn existing_next(&mut self) -> Option<X> {
            if self.index == self.vec.len() {
                None
            } else {
                self.index += 1;
                Some(self.vec[self.index - 1])
            }
        }
        fn next(&mut self, _domain: &[T; DIMENSIONS]) -> X {
            if self.index == self.vec.len() {
                self.index = 0;
            }
            self.index += 1;
            self.vec[self.index - 1]
        }

        fn restart(&mut self) {
            self.index = 0;
        }

        fn size(&self) -> usize {
            self.vec.len()
        }
    }
}
