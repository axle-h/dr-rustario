use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Formatter};
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand::distributions::Standard;
use rand_chacha::{ChaCha8Rng, ChaChaRng};
use crate::game::block::Block;
use crate::game::bottle::{Bottle, BOTTLE_FLOOR, BOTTLE_HEIGHT, BOTTLE_WIDTH, TOTAL_BLOCKS};
use crate::game::geometry::BottlePoint;
use crate::game::pill::{PillShape, VirusColor};

pub const PEEK_SIZE: usize = 5;
pub const MAX_BOTTLE_SEED_ATTEMPTS: usize = 100_000;

type Seed = <ChaCha8Rng as SeedableRng>::Seed;

impl Distribution<VirusColor> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> VirusColor {
        rng.gen_range(0..VirusColor::N).try_into().unwrap()
    }
}

impl Distribution<PillShape> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PillShape {
        PillShape::new(rng.gen(), rng.gen())
    }
}

pub fn random(count: usize) -> Vec<GameRandom> {
    let mut seed: Seed = Default::default();
    thread_rng().fill(&mut seed);
    (0..count)
        .map(|_| GameRandom::from_seed(seed))
        .collect()
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BottleSeed {
    viruses: [Option<VirusColor>; TOTAL_BLOCKS as usize],
    count: u32
}

impl BottleSeed {
    pub fn new() -> Self {
        Self { viruses: [None; TOTAL_BLOCKS as usize], count: 0 }
    }

    pub fn into_blocks(self) -> [Block; TOTAL_BLOCKS as usize] {
        self.viruses.map(|c| match c {
            Some(color) => Block::Virus(color),
            None => Block::Empty
        })
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    fn get(&self, x: i32, y: i32) -> Option<VirusColor> {
        if x >= 0 && x < BOTTLE_WIDTH as i32 && y >=0 && y < BOTTLE_HEIGHT as i32 {
            self.viruses[BottleSeed::index(x, y)]
        } else {
            None
        }
    }

    fn get_available_colors(&self, x: i32, y: i32) -> HashSet<VirusColor> {
        let mut colors = HashSet::from_iter([
            VirusColor::Yellow, VirusColor::Blue, VirusColor::Red
        ]);

        // 3-Consecutive Rule: viruses of the same color cannot occupy three consecutive cells in the same row or column
        if let Some(color) = self.get(x - 1, y) {
            if self.get(x + 1, y) == Some(color) {
                colors.remove(&color);
            }
        }
        if let Some(color) = self.get(x, y - 1) {
            if self.get(x, y + 1) == Some(color) {
                colors.remove(&color);
            }
        }

        // 2-Away Rule: viruses of the same color cannot occupy cells at distance two that are on the same row or column
        self.get(x, y - 2).map(|c| colors.remove(&c)); // top
        self.get(x + 2, y).map(|c| colors.remove(&c)); // right
        self.get(x, y + 2).map(|c| colors.remove(&c)); // bottom
        self.get(x - 2, y).map(|c| colors.remove(&c)); // left

        colors
    }

    fn set(&mut self, x: i32, y: i32, color: VirusColor) {
        let index = BottleSeed::index(x, y);
        assert!(self.viruses[index].is_none());
        self.count += 1;
        self.viruses[index] = Some(color);
    }

    fn index(x: i32, y: i32) -> usize {
        (y * BOTTLE_WIDTH as i32 + x) as usize
    }
}

impl Debug for BottleSeed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "   {}", "-".repeat(BOTTLE_WIDTH as usize))?;
        for y in 0..BOTTLE_HEIGHT {
            write!(f, "{:02}|", y)?;
            for x in 0..BOTTLE_WIDTH {
                match self.get(x as i32, y as i32) {
                    Some(color) => write!(f, "{}", color.to_char().to_ascii_uppercase())?,
                    None => write!(f, " ")?,
                }
            }
            writeln!(f, "|")?;
        }
        write!(f, "   {}", "-".repeat(BOTTLE_WIDTH as usize))
    }
}

pub struct GameRandom {
    pill_rng: ChaChaRng,
    bottle_rng: ChaChaRng,
    queue: VecDeque<PillShape>
}

impl GameRandom {
    pub fn from_seed(seed: Seed) -> Self {
        Self::new(ChaChaRng::from_seed(seed))
    }

    #[cfg(test)]
    pub fn from_u64_seed(seed: u64) -> Self {
        Self::new(ChaChaRng::seed_from_u64(seed))
    }

    pub fn new(rng: ChaChaRng) -> Self {
        let bottle_rng = rng.clone();
        let mut pill_rng = rng;
        let queue = (0..PEEK_SIZE)
            .map(|_| pill_rng.gen())
            .collect();
        Self { pill_rng, bottle_rng, queue }
    }

    pub fn peek(&self) -> [PillShape; PEEK_SIZE] {
        self.queue
            .iter()
            .take(PEEK_SIZE)
            .copied()
            .collect::<Vec<PillShape>>()
            .try_into()
            .unwrap()
    }

    pub fn next_pill(&mut self) -> PillShape {
        self.queue.push_back(self.pill_rng.gen());
        self.queue.pop_front().unwrap()
    }

    pub fn bottle_seed(&mut self, virus_level: u32) -> Result<BottleSeed, String> {
        for _ in 0..MAX_BOTTLE_SEED_ATTEMPTS {
            if let Some(seed) = self.try_bottle_seed(virus_level) {
                return Ok(seed);
            }
        }
        Err(format!("failed to generate valid bottle after {} attempts", MAX_BOTTLE_SEED_ATTEMPTS))
    }

    fn try_bottle_seed(&mut self, virus_level: u32) -> Option<BottleSeed> {
        let mut bottle = BottleSeed::new();
        let target = (virus_level * 4 + 4).min(99);
        let max_virus_row = match virus_level {
            0..=14 => 6,
            15 | 16 => 5,
            17 | 18 => 4,
            _ => 3
        };
        let mut available = (max_virus_row..BOTTLE_HEIGHT)
            .flat_map(|y| (0..BOTTLE_WIDTH).map(move |x| BottlePoint::new(x as i32, y as i32)))
            .collect::<Vec<BottlePoint>>();
        available.shuffle(&mut self.pill_rng);

        for i in 0..target {
            let point = available.pop()?;
            let available_colors = bottle.get_available_colors(point.x(), point.y());
            if available_colors.is_empty() {
                continue;
            }
            let mut color: VirusColor = match (i as usize % 4).try_into() {
                Ok(color) => color,
                _ => self.pill_rng.gen()
            };
            while !available_colors.contains(&color) {
                color = color.next();
            }
            bottle.set(point.x(), point.y(), color);
        }

        if bottle.count < target {
            None
        } else {
            Some(bottle)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_bottle_at_level_0() {
        let mut source = GameRandom::from_u64_seed(123546);
        let result = source.bottle_seed(0).expect("should generate valid bottle for this seed");
        assert_eq!(result.virus_count(), 4, "{:?}", result);
    }

    #[test]
    fn seeds_bottle_with_99_viruses() {
        let mut source = GameRandom::from_u64_seed(123546);
        let result = source.bottle_seed(30).expect("should generate valid bottle for this seed");
        assert_eq!(result.virus_count(), 99, "{:?}", result);
        println!("{:?}", result);

        // revalidate all placements
        for x in 0..BOTTLE_WIDTH as i32  {
            for y in 0 ..BOTTLE_HEIGHT as i32 {
                if let Some(color) = result.get(x, y) {
                    assert!(result.get_available_colors(x, y).contains(&color));
                }
            }
        }

        // validate 3 in a row rule
        for color in [VirusColor::Yellow, VirusColor::Red, VirusColor::Blue] {
            // horizontal
            let mut count = 0;
            for y in 0 ..BOTTLE_HEIGHT as i32 {
                count = 0;
                for x in 0..BOTTLE_WIDTH as i32 {
                    if result.get(x, y) == Some(color) {
                        count += 1;
                    } else {
                        count = 0;
                    }
                    assert!(count < 3, "({}, {}) {:?}", x, y, result);
                }
            }

            // vertical
            let mut count = 0;
            for x in 0..BOTTLE_WIDTH as i32  {
                count = 0;
                for y in 0 ..BOTTLE_HEIGHT as i32 {
                    if result.get(x, y) == Some(color) {
                        count += 1;
                    } else {
                        count = 0;
                    }
                    assert!(count < 3, "({}, {}) {:?}", x, y, result);
                }
            }
        }
    }

    trait BottleSeedTestHarness {
        fn virus_count(&self) -> usize;
    }

    impl BottleSeedTestHarness for BottleSeed {
        fn virus_count(&self) -> usize {
            self.viruses.iter().filter(|x| x.is_some()).count()
        }
    }
}