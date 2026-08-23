use super::tetromino::TetrominoShape;
use crate::game::board::BOARD_WIDTH;
pub use engine::game::random::RandomMode;
use engine::game::random::BagRandom;
#[cfg(feature = "ai")]
use num_bigint::BigUint;
#[cfg(feature = "ai")]
use num_traits::Num;
use rand::distr::StandardUniform;
use rand::prelude::*;
use rand::{Rng, RngExt};
use rand_chacha::ChaChaRng;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Deref, DerefMut};

pub const PEEK_SIZE: usize = 7;
/// the garbage hole moves every n rows of garbage
pub const MIN_GARBAGE_PER_HOLE: u32 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seed(<ChaChaRng as SeedableRng>::Seed);

impl Deref for Seed {
    type Target = <ChaChaRng as SeedableRng>::Seed;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Seed {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Seed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "ai")]
        {
            let bigint: BigUint = (*self).into();
            write!(f, "{}", bigint)
        }
        #[cfg(not(feature = "ai"))]
        {
            write!(f, "{}", engine::game::random::Seed::from(self.0))
        }
    }
}

impl Add for Seed {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl AddAssign for Seed {
    fn add_assign(&mut self, rhs: Self) {
        let mut carry = 0u64;

        // Process 8 bytes at a time using u64
        for i in (0..32).step_by(8) {
            let a = u64::from_le_bytes(self[i..i+8].try_into().unwrap());
            let b = u64::from_le_bytes(rhs[i..i+8].try_into().unwrap());

            // Add previous carry to first number
            let sum = a.wrapping_add(b).wrapping_add(carry);

            // Calculate new carry - if sum is less than either input (or equal to when carry is 1),
            // we wrapped around and need to carry 1
            carry = if (carry == 1 && sum <= a) || (carry == 0 && sum < a) {
                1
            } else {
                0
            };

            self[i..i+8].copy_from_slice(&sum.to_le_bytes());
        }
    }
}

impl Default for Seed {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl Distribution<Seed> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Seed {
        Seed(rng.random())   
    }
}

impl From<u128> for Seed {
    fn from(value: u128) -> Self {
        let mut seed = Seed::default();
        seed[..16].copy_from_slice(&value.to_le_bytes());
        seed
    }
}

#[cfg(feature = "ai")]
impl From<BigUint> for Seed {
    fn from(value: BigUint) -> Self {
        let mut bytes = value.to_bytes_be();
        // pad to 32 bytes
        while bytes.len() < 32 {
            bytes.insert(0, 0);
        }
        Self(bytes.try_into().expect("expecting a 256 bit number"))
    }
}

#[cfg(feature = "ai")]
impl Into<BigUint> for Seed {
    fn into(self) -> BigUint {
        BigUint::from_bytes_be(&*self)
    }
}

impl From<i32> for Seed {
    fn from(value: i32) -> Self {
        let mut seed = Seed::default();
        seed[..4].copy_from_slice(&value.to_le_bytes());
        seed
    }
}

#[cfg(feature = "ai")]
impl From<String> for Seed {
    fn from(value: String) -> Self {
        BigUint::from_str_radix(&value, 10).expect("not a valid seed string").into()
    }
}

impl Into<ChaChaRng> for Seed {
    fn into(self) -> ChaChaRng {
        ChaChaRng::from_seed(self.0)
    }
}

impl From<Seed> for engine::game::random::Seed {
    fn from(seed: Seed) -> Self {
        Self::from(seed.0)
    }
}

pub fn random_tetrominos(mode: RandomMode, count: usize) -> Vec<RandomTetromino> {
    let seed: Seed = rand::random();
    (0..count)
        .map(|_| RandomTetromino::new(mode, MIN_GARBAGE_PER_HOLE, seed))
        .collect()
}

#[derive(Clone, Debug)]
pub struct RandomTetromino {
    min_garbage_per_hole: u32, // move the garbage hole every n garbage
    garbage_since_last_hole: u32,
    current_garbage_hole: u32,
    shapes: BagRandom<TetrominoShape>,
}

impl RandomTetromino {
    pub fn new(random_mode: RandomMode, min_garbage_per_hole: u32, seed: Seed) -> Self {
        let mut rng: ChaChaRng = seed.into();
        let current_garbage_hole = rng.random_range(0..BOARD_WIDTH);
        Self {
            min_garbage_per_hole,
            garbage_since_last_hole: 0,
            current_garbage_hole,
            shapes: BagRandom::new(rng, random_mode, &TetrominoShape::ALL, PEEK_SIZE),
        }
    }

    pub fn next_garbage_hole(&mut self) -> u32 {
        let result = self.current_garbage_hole;
        self.garbage_since_last_hole += 1;
        if self.garbage_since_last_hole >= self.min_garbage_per_hole {
            self.garbage_since_last_hole = 0;
            self.current_garbage_hole = self.shapes.rng().random_range(0..BOARD_WIDTH);
        }
        result
    }

    pub fn next(&mut self) -> TetrominoShape {
        self.shapes.next()
    }

    pub fn peek_buffer(&self) -> [TetrominoShape; PEEK_SIZE] {
        self.shapes.peek().try_into().unwrap()
    }

    pub fn peek(&self) -> TetrominoShape {
        self.shapes.peek_next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn next_n(random: &mut RandomTetromino, n: usize) -> Vec<TetrominoShape> {
        (0..n).map(|_| random.next()).collect()
    }

    fn next_n_holes(random: &mut RandomTetromino, n: usize) -> Vec<u32> {
        (0..n).map(|_| random.next_garbage_hole()).collect()
    }

    #[test]
    fn bag_random() {
        let mut random = RandomTetromino::new(RandomMode::Bag, 10, rand::random());

        // chunk into 3 bags of 7 shapes (arrays make it easier for creating the sets)
        let bags: Vec<[TetrominoShape; 7]> = next_n(&mut random, 21)
            .chunks(7)
            .map(|chunk| chunk.to_vec().try_into().unwrap())
            .collect();

        // each bag should not be in same order
        assert_ne!(bags[0], bags[1]);
        assert_ne!(bags[1], bags[2]);

        // but should all contain all the shapes
        let all_shapes = HashSet::from(TetrominoShape::ALL);
        assert_eq!(HashSet::from(bags[0]), all_shapes);
        assert_eq!(HashSet::from(bags[1]), all_shapes);
        assert_eq!(HashSet::from(bags[2]), all_shapes);
    }

    #[test]
    fn bag_random_peek() {
        let mut random = RandomTetromino::new(RandomMode::Bag, 10, rand::random());
        let peek = random.peek_buffer();
        let observed: [TetrominoShape; PEEK_SIZE] =
            next_n(&mut random, PEEK_SIZE).try_into().unwrap();
        assert_eq!(observed, peek);
    }

    #[test]
    fn true_random() {
        let mut random = RandomTetromino::new(RandomMode::True, 10, rand::random());
        let observed: [TetrominoShape; 1000] = next_n(&mut random, 1000).try_into().unwrap();
        // should generate all shapes in 1000 tries
        assert_eq!(HashSet::from(observed), HashSet::from(TetrominoShape::ALL));
    }

    #[test]
    fn true_random_peek() {
        let mut random = RandomTetromino::new(RandomMode::True, 10, rand::random());
        let peek = random.peek_buffer();
        let observed: [TetrominoShape; PEEK_SIZE] =
            next_n(&mut random, PEEK_SIZE).try_into().unwrap();
        assert_eq!(observed, peek);
    }

    #[test]
    fn static_garbage_hole() {
        let mut random = RandomTetromino::new(RandomMode::True, 100, rand::random());
        let observed: [u32; 100] = next_n_holes(&mut random, 100).try_into().unwrap();
        assert_eq!(HashSet::from(observed).len(), 1);
    }

    #[test]
    fn dynamic_garbage_hole() {
        let mut random = RandomTetromino::new(RandomMode::True, 1, rand::random());
        let observed: [u32; 100] = next_n_holes(&mut random, 100).try_into().unwrap();
        assert!(HashSet::from(observed).len() > 1);
    }
    
    #[test]
    fn sum_seed() {
        let seed1 = Seed::from(999999999999999999999999999u128);
        let seed2 = Seed::from(1);
        let seed3 = seed1 + seed2;
        assert_eq!(seed3, Seed::from(1000000000000000000000000000u128));
    }
    
    #[cfg(feature = "ai")]
    #[test]
    fn serialize_seed() {
        let bigint = BigUint::parse_bytes(b"111000000000000000000000000000000000000222", 10).unwrap();
        let result: BigUint = Seed::from(bigint.clone()).into(); 
        assert_eq!(result, bigint);
    }

    #[cfg(feature = "ai")]
    #[test]
    fn display_seed() {
        let bigint = BigUint::parse_bytes(b"34028236692093846346337460743176821145500000000000000000000000000000000000000", 10).unwrap();
        let result = format!("{}", Seed::from(bigint.clone()));
        assert_eq!(result, "34028236692093846346337460743176821145500000000000000000000000000000000000000");
    }

    #[cfg(feature = "ai")]
    #[test]
    fn parse_seed() {
        let seed: Seed = rand::random();
        let mut rng1 = ChaChaRng::from_seed(seed.0);
        let mut rng2 = ChaChaRng::from_seed(seed.0);

        let next1: u128 = rng1.random();
        let next2: u128 = rng2.random();

        assert_eq!(next1, next2, "RNGs should produce the same sequence with the same seed");
    }
}
