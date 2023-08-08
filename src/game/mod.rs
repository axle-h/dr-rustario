use std::collections::HashSet;
use std::time::Duration;
use crate::game::block::Block;
use crate::game::bottle::SendGarbage;
use crate::game::event::GameEvent;
use crate::game::geometry::BottlePoint;
use crate::game::pill::{PillShape, VirusColor};
use crate::game::random::GameRandom;

#[cfg(not(test))]
use crate::game::bottle::Bottle;
#[cfg(test)]
use crate::game::tests::MockBottle as Bottle;

pub mod pill;
pub mod geometry;
pub mod bottle;
pub mod block;
pub mod random;
mod event;

const SOFT_DROP_STEP_FACTOR: u32 = 20;
const SOFT_DROP_SPAWN_FACTOR: u32 = 10;
const GARBAGE_DROP_DURATION: Duration = Duration::from_millis(200);
const MIN_SPAWN_DELAY: Duration = Duration::from_millis(500);
const LOCK_DURATION: Duration = Duration::from_millis(500);
const SOFT_DROP_LOCK_DURATION: Duration = Duration::from_millis(500 / 2);
const MAX_LOCK_PLACEMENTS: u32 = 15;

const SPEED_TABLE: [Duration; 81] = [
    Duration::from_nanos(1166666667),
    Duration::from_nanos(1133333333),
    Duration::from_nanos(1100000000),
    Duration::from_nanos(1066666667),
    Duration::from_nanos(1033333333),
    Duration::from_nanos(1000000000),
    Duration::from_nanos(966666667),
    Duration::from_nanos(933333333),
    Duration::from_nanos(900000000),
    Duration::from_nanos(866666667),
    Duration::from_nanos(833333333),
    Duration::from_nanos(800000000),
    Duration::from_nanos(766666667),
    Duration::from_nanos(733333333),
    Duration::from_nanos(700000000),
    Duration::from_nanos(666666667),
    Duration::from_nanos(633333333),
    Duration::from_nanos(600000000),
    Duration::from_nanos(566666667),
    Duration::from_nanos(533333333),
    Duration::from_nanos(500000000),
    Duration::from_nanos(466666667),
    Duration::from_nanos(433333333),
    Duration::from_nanos(400000000),
    Duration::from_nanos(366666667),
    Duration::from_nanos(333333333),
    Duration::from_nanos(316666667),
    Duration::from_nanos(300000000),
    Duration::from_nanos(283333333),
    Duration::from_nanos(266666667),
    Duration::from_nanos(250000000),
    Duration::from_nanos(233333333),
    Duration::from_nanos(216666667),
    Duration::from_nanos(200000000),
    Duration::from_nanos(183333333),
    Duration::from_nanos(166666667),
    Duration::from_nanos(166666667),
    Duration::from_nanos(150000000),
    Duration::from_nanos(150000000),
    Duration::from_nanos(133333333),
    Duration::from_nanos(133333333),
    Duration::from_nanos(116666667),
    Duration::from_nanos(116666667),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(100000000),
    Duration::from_nanos(83333333),
    Duration::from_nanos(83333333),
    Duration::from_nanos(83333333),
    Duration::from_nanos(83333333),
    Duration::from_nanos(83333333),
    Duration::from_nanos(66666667),
    Duration::from_nanos(66666667),
    Duration::from_nanos(66666667),
    Duration::from_nanos(66666667),
    Duration::from_nanos(66666667),
    Duration::from_nanos(50000000),
    Duration::from_nanos(50000000),
    Duration::from_nanos(50000000),
    Duration::from_nanos(50000000),
    Duration::from_nanos(50000000),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(33333333),
    Duration::from_nanos(16666667),
];

pub enum GameSpeed {
    Low,
    Medium,
    High
}

impl GameSpeed {
    const MAX_LEVEL: usize = 49;

    fn drop_duration(&self, total_pills: usize) -> Duration {
        self.duration_of_level(total_pills / 10)
    }

    fn min_drop_duration(&self) -> Duration {
        self.duration_of_level(Self::MAX_LEVEL)
    }

    fn duration_of_level(&self, speed_level: usize) -> Duration {
        let index = match self {
            GameSpeed::Low => 15,
            GameSpeed::Medium => 25,
            GameSpeed::High => 31,
        } + speed_level.min(Self::MAX_LEVEL);
        SPEED_TABLE[index]
    }

    fn base_index(&self) -> usize {
        match self {
            GameSpeed::Low => 15,
            GameSpeed::Medium => 25,
            GameSpeed::High => 31,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    Spawn(Duration),
    SpawnHold(PillShape),
    Fall(Duration),
    Lock(Duration),
    /// check the bottle for patterns to destroy
    Pattern,
    /// destroy marked patterns
    Destroy(HashSet<BottlePoint>),
    GameOver,
    DropGarbage(Duration),
}

impl GameState {
    const NEW_LOCK: Self = Self::Lock(Duration::ZERO);
    const LOCK_NOW: Self = Self::Lock(LOCK_DURATION);
    const NEW_FALL: Self = Self::Fall(Duration::ZERO);
    const NEW_SPAWN: Self = Self::Spawn(Duration::ZERO);
    const NEW_DROP_GARBAGE: Self = Self::DropGarbage(Duration::ZERO);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HoldState {
    shape: PillShape,
    locked: bool,
}

impl HoldState {
    pub fn locked(shape: PillShape) -> Self {
        Self { shape, locked: true }
    }
}

pub struct Game {
    player: u32,
    virus_level: u32,
    speed_level: GameSpeed,
    random: GameRandom,
    events: Vec<GameEvent>,
    bottle: Bottle,
    state: GameState,
    total_pills: usize,
    soft_drop: bool,
    hard_dropped: bool,
    hold: Option<HoldState>,
    garbage_buffer: Vec<SendGarbage>,
}

impl Game {
    pub fn new(player: u32, virus_level: u32, speed_level: GameSpeed, mut random: GameRandom) -> Result<Self, String> {
        let bottle = Bottle::from_seed(random.bottle_seed(virus_level)?);
        Ok(Self::from_bottle(player, virus_level, speed_level, random, bottle))
    }

    pub fn from_bottle(player: u32, virus_level: u32, speed_level: GameSpeed, mut random: GameRandom, bottle: Bottle) -> Self {
        Self {
            player,
            virus_level,
            speed_level,
            random,
            events: vec![],
            bottle,
            state: GameState::NEW_SPAWN,
            total_pills: 0,
            soft_drop: false,
            hard_dropped: false,
            hold: None,
            garbage_buffer: vec![],
        }
    }

    pub fn row(&self, y: u32) -> &[Block] {
        self.bottle.row(y)
    }

    pub fn hold(&mut self) {
        if matches!(self.hold, Some(HoldState { locked: true, .. })) {
            // hold is blocked
            return;
        }

        let held_shape = match self.bottle.hold() {
            None => return,
            Some(shape) => shape,
        };

        let shape = match self.hold {
            None => self.random.next_pill(), // just spawn next random shape
            Some(HoldState { shape, .. }) => shape,
        };

        self.state = GameState::SpawnHold(shape);
        self.hold = Some(HoldState::locked(held_shape));
        self.events.push(GameEvent::Hold);
    }

    pub fn set_soft_drop(&mut self, soft_drop: bool) {
        self.soft_drop = soft_drop;
        if soft_drop {
            self.events.push(GameEvent::SoftDrop);
        }
    }

    pub fn hard_drop(&mut self) {
        if let Some((dropped_rows, vitamins)) = self.bottle.hard_drop() {
            self.state = GameState::LOCK_NOW;
            self.hard_dropped = true;
            self.events.push(
                GameEvent::HardDrop { player: self.player, dropped_rows, vitamins }
            );
        }
    }

    pub fn left(&mut self) {
        if self.with_checking_lock(|bottle| bottle.left()) {
            self.events.push(GameEvent::Move);
        }
    }

    pub fn right(&mut self) {
        if self.with_checking_lock(|bottle| bottle.right()) {
            self.events.push(GameEvent::Move);
        }
    }

    pub fn rotate(&mut self, clockwise: bool) {
        if self.with_checking_lock(|bottle| bottle.rotate(clockwise)) {
            self.events.push(GameEvent::Rotate);
        }
    }

    pub fn send_garbage(&mut self, garbage: SendGarbage) {
        self.garbage_buffer.push(garbage);
    }

    pub fn update(&mut self, delta: Duration) {
        self.state = match &self.state {
            GameState::Spawn(duration) => self.next_spawn(*duration + delta),
            GameState::SpawnHold(shape) => self.spawn_shape(*shape),
            GameState::Fall(duration) => self.next_fall(*duration + delta),
            GameState::Lock(duration) => self.next_lock(*duration + delta),
            GameState::Pattern => self.next_pattern(),
            GameState::Destroy(points) => self.next_destroy(points.clone()),
            GameState::GameOver => GameState::GameOver,
            GameState::DropGarbage(duration) => self.next_drop_garbage(*duration + delta),
        };
    }

    fn next_spawn(&mut self, duration: Duration) -> GameState {
        if let Some(next_garbage) = self.garbage_buffer.pop() {
            let garbage = self.bottle.send_garbage(next_garbage);
            self.events.push(GameEvent::ReceivedGarbage { player: self.player, garbage });
            return GameState::NEW_DROP_GARBAGE;
        }

        if !self.hard_dropped && duration < self.spawn_delay() {
            return GameState::Spawn(duration);
        }
        self.hard_dropped = false;
        let shape = self.random.next_pill();
        dbg!(shape);
        self.spawn_shape(shape)
    }

    fn spawn_shape(&mut self, shape: PillShape) -> GameState {
        if let Some(vitamins) = self.bottle.try_spawn(shape) {
            self.events.push(
                GameEvent::Spawn {
                    player: self.player,
                    vitamins,
                }
            );
            GameState::NEW_FALL
        } else {
            // cannot spawn a pill is a game over event
            self.events.push(GameEvent::GameOver { player: self.player });
            GameState::GameOver
        }
    }

    fn next_fall(&mut self, duration: Duration) -> GameState {
        if duration < self.step_delay() {
            return GameState::Fall(duration);
        }

        if !self.bottle.step_down_pill() {
            // cannot step down, start lock
            return GameState::NEW_LOCK;
        }

        self.events.push(GameEvent::Fall);
        if self.bottle.is_collision() {
            // step has caused a collision, start a lock
            if self.bottle.lock_placements() >= MAX_LOCK_PLACEMENTS {
                GameState::LOCK_NOW
            } else {
                GameState::NEW_LOCK
            }
        } else {
            // no collisions, start a new fall step
            GameState::NEW_FALL
        }
    }

    fn next_lock(&mut self, duration: Duration) -> GameState {
        let max_lock_duration = if self.soft_drop {
            SOFT_DROP_LOCK_DURATION
        } else {
            LOCK_DURATION
        };
        if !self.hard_dropped && duration < max_lock_duration {
            GameState::Lock(duration)
        } else if self.bottle.is_collision() {
            // lock timeout and still colliding so lock the piece now
            // but before locking, need to check for a game over event.
            let vitamins = self.bottle.lock().expect("we must've locked");

            // maybe unlock hold
            if let Some(hold) = self.hold.as_mut() {
                hold.locked = false;
            }

            self.events.push(
                GameEvent::Lock {
                    player: self.player,
                    vitamins,
                    hard_or_soft_dropped: self.hard_dropped || self.soft_drop,
                }
            );
            GameState::Pattern
        } else {
            // otherwise must've moved over empty space so start a new fall
            GameState::NEW_FALL
        }
    }

    fn next_pattern(&mut self) -> GameState {
        let points = self.bottle.pattern();
        self.events.push(GameEvent::Destroy(points.clone()));
        GameState::Destroy(points)
    }

    fn next_destroy(&mut self, points: HashSet<BottlePoint>) -> GameState {
        self.bottle.destroy(points);
        // todo combo -> garbage & score
        GameState::NEW_DROP_GARBAGE
    }

    fn next_drop_garbage(&mut self, duration: Duration) -> GameState {
        if duration < GARBAGE_DROP_DURATION {
            return GameState::DropGarbage(duration);
        }

        if self.bottle.step_down_garbage() {
            // garbage dropped so try again
            self.events.push(GameEvent::DropGarbage);
            GameState::NEW_DROP_GARBAGE
        } else {
            // no garbage to drop so spawn a new pill
            GameState::NEW_SPAWN
        }
    }

    fn with_checking_lock<F>(&mut self, mut f: F) -> bool
        where
            F: FnMut(&mut Bottle) -> bool,
    {
        if let GameState::Lock(lock_duration) = self.state {
            // 1. check if the lock is already breached (we send movements before a lock update)
            if lock_duration >= LOCK_DURATION {
                return false;
            }
            // 2. check if this pill used all it's lock movements for this altitude
            if self.bottle.lock_placements() >= MAX_LOCK_PLACEMENTS {
                // the pill has already run out of lock movements, lock it asap
                self.state = GameState::LOCK_NOW;
                return false;
            }
            // 3. check the movement was blocked by the board
            if !f(&mut self.bottle) {
                return false;
            }
            if self.bottle.register_lock_placement() < MAX_LOCK_PLACEMENTS {
                // movement is allowed under lock, lock is reset
                self.state = GameState::NEW_FALL;
            } else {
                // the pill just ran out of lock movements, lock it asap
                self.state = GameState::LOCK_NOW;
            }
            true
        } else {
            // not in lock state, pass through closure
            f(&mut self.bottle)
        }
    }

    fn spawn_delay(&self) -> Duration {
        self.base_delay(SOFT_DROP_SPAWN_FACTOR).max(MIN_SPAWN_DELAY)
    }

    fn step_delay(&self) -> Duration {
        self.base_delay(SOFT_DROP_STEP_FACTOR)
    }

    fn base_delay(&self, soft_drop_factor: u32) -> Duration {
        let base = self.speed_level.drop_duration(self.total_pills);
        if self.soft_drop {
            (base / soft_drop_factor).max(self.speed_level.min_drop_duration())
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use mockall::predicate::*;
    use rand::random;
    use crate::game::pill::Vitamin;
    use super::random::BottleSeed;
    use super::pill::{Vitamins, Garbage};
    use super::*;

    mock! {
        pub Bottle {
            pub fn from_seed(seed: BottleSeed) -> Self;
            pub fn row(&self, y: u32) -> &[Block];
            pub fn left(&mut self) -> bool;
            pub fn right(&mut self) -> bool;
            pub fn rotate(&mut self, clockwise: bool) -> bool;
            pub fn hold(&mut self) -> Option<PillShape>;
            pub fn hard_drop(&mut self) -> Option<(u32, Vitamins)>;
            pub fn register_lock_placement(&mut self) -> u32;
            pub fn lock_placements(&self) -> u32;
            pub fn is_collision(&self) -> bool;
            pub fn send_garbage(&mut self, garbage: SendGarbage) -> Vec<Garbage>;
            pub fn try_spawn(&mut self, shape: PillShape) -> Option<Vitamins>;
            pub fn step_down_pill(&mut self) -> bool;
            pub fn lock(&mut self) -> Option<Vitamins>;
            pub fn pattern(&self) -> HashSet<BottlePoint>;
            pub fn destroy(&mut self, points: HashSet<BottlePoint>);
            pub fn step_down_garbage(&mut self) -> bool;
        }
    }

    #[test]
    fn left_success() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_left().return_once(|| true);
        });
        game.left();
        game.should_have_events(&[GameEvent::Move]);
    }

    #[test]
    fn left_fail() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_left().return_once(|| false);
        });
        game.left();
        game.should_have_no_events();
    }

    #[test]
    fn right_success() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_right().return_once(|| true);
        });
        game.right();
        game.should_have_events(&[GameEvent::Move]);
    }

    #[test]
    fn right_fail() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_right().return_once(|| false);
        });
        game.right();
        game.should_have_no_events();
    }

    #[test]
    fn rotate_success_when_falling() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_rotate().with(eq(true)).return_once(|_| true);
        });
        game.state = GameState::NEW_FALL;
        game.rotate(true);
        game.should_have_events(&[GameEvent::Rotate]);
    }

    #[test]
    fn rotate_success_and_reset_lock_when_locking() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_rotate().with(eq(true)).return_once(|_| true);
            bottle.expect_lock_placements().return_once(|| 0);
            bottle.expect_register_lock_placement().return_once(|| 1);
        });
        game.state = GameState::Lock(Duration::from_millis(10));
        game.rotate(true);
        game.should_have_events(&[GameEvent::Rotate]);
        assert_eq!(game.state, GameState::NEW_FALL);
    }

    #[test]
    fn rotate_success_and_lock_with_last_lock_placement() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_rotate().with(eq(true)).return_once(|_| true);
            bottle.expect_lock_placements().return_once(|| MAX_LOCK_PLACEMENTS - 1);
            bottle.expect_register_lock_placement().return_once(|| MAX_LOCK_PLACEMENTS);
        });
        game.state = GameState::Lock(Duration::from_millis(10));
        game.rotate(true);
        game.should_have_events(&[GameEvent::Rotate]);
        assert_eq!(game.state, GameState::LOCK_NOW);
    }

    #[test]
    fn rotate_fail() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_rotate().with(eq(true)).return_once(|_| false);
        });
        game.rotate(true);
        game.should_have_no_events();
    }

    #[test]
    fn rotate_fail_when_locked() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_rotate().with(eq(true)).return_once(|_| true);
        });
        game.state = GameState::LOCK_NOW;
        game.rotate(true);
        game.should_have_no_events();
        assert_eq!(game.state, GameState::LOCK_NOW);
    }

    #[test]
    fn rotate_fail_when_no_lock_placements_left() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_rotate().with(eq(true)).return_once(|_| true);
            bottle.expect_lock_placements().return_once(|| MAX_LOCK_PLACEMENTS);
        });
        game.state = GameState::NEW_LOCK;
        game.rotate(true);
        game.should_have_no_events();
        assert_eq!(game.state, GameState::LOCK_NOW);
    }

    #[test]
    fn holds_for_first_time() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_hold().return_once(|| Some(PillShape::RED_BLUE));
        });
        game.hold();
        game.should_have_events(&[GameEvent::Hold]);
        assert_eq!(game.state, GameState::SpawnHold(PillShape::BLUE_BLUE));
        assert_eq!(game.hold, Some(HoldState::locked(PillShape::RED_BLUE)))
    }

    #[test]
    fn holds_for_second_time() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_hold().return_once(|| Some(PillShape::RED_BLUE));
        });
        game.hold = Some(HoldState { shape: PillShape::RED_RED, locked: false });
        game.hold();
        game.should_have_events(&[GameEvent::Hold]);
        assert_eq!(game.state, GameState::SpawnHold(PillShape::RED_RED));
        assert_eq!(game.hold, Some(HoldState::locked(PillShape::RED_BLUE)))
    }

    #[test]
    fn cannot_hold_when_hold_locked() {
        let mut game = having_bottle(|_| {});
        game.state = GameState::NEW_FALL;
        game.hold = Some(HoldState::locked(PillShape::RED_BLUE));
        game.hold();
        game.should_have_no_events();
        assert_eq!(game.state, GameState::NEW_FALL);
    }

    #[test]
    fn cannot_hold_when_bottle_rejected() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_hold().return_once(|| None);
        });
        game.state = GameState::NEW_FALL;
        game.hold();
        game.should_have_no_events();
        assert_eq!(game.state, GameState::NEW_FALL);
    }

    #[test]
    fn soft_drop_on() {
        let mut game = having_bottle(|_| {});
        game.set_soft_drop(true);
        game.should_have_events(&[GameEvent::SoftDrop]);
    }

    #[test]
    fn soft_drop_off() {
        let mut game = having_bottle(|_| {});
        game.set_soft_drop(false);
        game.should_have_no_events();
    }

    #[test]
    fn hard_drop_success() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_hard_drop().return_once(|| Some((10, Vitamin::vitamins(PillShape::RED_BLUE))));
        });
        game.hard_drop();
        game.should_have_events(&[GameEvent::HardDrop {
            player: 0,
            vitamins: Vitamin::vitamins(PillShape::RED_BLUE),
            dropped_rows: 10
        }]);
        assert_eq!(game.state, GameState::LOCK_NOW);
        assert!(game.hard_dropped)
    }

    #[test]
    fn hard_drop_fail() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_hard_drop().return_once(|| None);
        });
        game.state = GameState::NEW_FALL;
        game.hard_drop();
        game.should_have_no_events();
        assert_eq!(game.state, GameState::NEW_FALL);
    }
    
    #[test]
    fn send_garbage() {
        let mut game = having_bottle(|_| {});
        game.send_garbage(vec![VirusColor::Red, VirusColor::Blue]);
        assert_eq!(game.garbage_buffer, vec![vec![VirusColor::Red, VirusColor::Blue]]);
    }

    #[test]
    fn update_spawn_into_spawn() {
        let mut game = having_bottle(|_| {});
        game.state = GameState::Spawn(Duration::from_nanos(1));
        game.update(Duration::from_nanos(2));
        assert_eq!(game.state,  GameState::Spawn(Duration::from_nanos(3)));
        game.should_have_no_events();
    }

    #[test]
    fn update_spawn_into_fall() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_try_spawn()
                .with(eq(PillShape::BLUE_BLUE))
                .return_once(|_| Some(Vitamin::vitamins(PillShape::BLUE_BLUE)));
        });
        game.state = GameState::Spawn(GameSpeed::Low.drop_duration(0));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_FALL);
        game.should_have_events(&[GameEvent::Spawn { player: 0, vitamins: Vitamin::vitamins(PillShape::BLUE_BLUE) }]);
    }

    #[test]
    fn update_hard_dropped_spawn_into_fall() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_try_spawn()
                .with(eq(PillShape::BLUE_BLUE))
                .return_once(|_| Some(Vitamin::vitamins(PillShape::BLUE_BLUE)));
        });
        game.hard_dropped = true;
        game.state = GameState::NEW_SPAWN;
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_FALL);
        assert!(!game.hard_dropped);
        game.should_have_events(&[GameEvent::Spawn { player: 0, vitamins: Vitamin::vitamins(PillShape::BLUE_BLUE) }]);
    }

    #[test]
    fn update_spawn_into_game_over() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_try_spawn()
                .with(eq(PillShape::BLUE_BLUE))
                .return_once(|_| None);
        });
        game.state = GameState::Spawn(GameSpeed::Low.drop_duration(0));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::GameOver);
        game.should_have_events(&[GameEvent::GameOver { player: 0 }]);
    }

    #[test]
    fn update_spawn_into_garbage() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_send_garbage()
                .with(eq(vec![VirusColor::Red, VirusColor::Yellow]))
                .return_once(|_| vec![Garbage::new(VirusColor::Yellow, BottlePoint::new(1, 2))]);
        });
        game.garbage_buffer.push(vec![VirusColor::Red, VirusColor::Yellow]);
        game.state = GameState::NEW_SPAWN;
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_DROP_GARBAGE);
        game.should_have_events(&[GameEvent::ReceivedGarbage {
            player: 0,
            garbage: vec![Garbage::new(VirusColor::Yellow, BottlePoint::new(1, 2))]
        }]);
    }

    #[test]
    fn update_hold_spawn_into_fall() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_try_spawn()
                .with(eq(PillShape::RED_BLUE))
                .return_once(|_| Some(Vitamin::vitamins(PillShape::RED_BLUE)));
        });
        game.state = GameState::SpawnHold(PillShape::RED_BLUE);
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_FALL);
        game.should_have_events(&[GameEvent::Spawn { player: 0, vitamins: Vitamin::vitamins(PillShape::RED_BLUE) }]);
    }

    #[test]
    fn update_fall_into_fall() {
        let mut game = having_bottle(|_| {});
        game.state = GameState::Fall(Duration::from_nanos(1));
        game.update(Duration::from_nanos(2));
        assert_eq!(game.state,  GameState::Fall(Duration::from_nanos(3)));
        game.should_have_no_events();
    }

    #[test]
    fn update_fall_into_next_fall() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_step_down_pill().return_once(|| true);
            bottle.expect_is_collision().return_once(|| false);
        });
        game.state = GameState::Fall(GameSpeed::Low.drop_duration(0));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state,  GameState::NEW_FALL);
        game.should_have_events(&[GameEvent::Fall]);
    }

    #[test]
    fn update_fall_into_lock_by_fail() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_step_down_pill().return_once(|| false);
        });
        game.state = GameState::Fall(GameSpeed::Low.drop_duration(0));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_LOCK);
        game.should_have_no_events();
    }

    #[test]
    fn update_fall_into_lock_by_collision() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_step_down_pill().return_once(|| true);
            bottle.expect_is_collision().return_once(|| true);
            bottle.expect_lock_placements().return_once(|| 0);
        });
        game.state = GameState::Fall(GameSpeed::Low.drop_duration(0));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_LOCK);
        game.should_have_events(&[GameEvent::Fall]);
    }

    #[test]
    fn update_fall_into_lock_asap_by_collision() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_step_down_pill().return_once(|| true);
            bottle.expect_is_collision().return_once(|| true);
            bottle.expect_lock_placements().return_once(|| MAX_LOCK_PLACEMENTS);
        });
        game.state = GameState::Fall(GameSpeed::Low.drop_duration(0));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::LOCK_NOW);
        game.should_have_events(&[GameEvent::Fall]);
    }

    #[test]
    fn update_lock_into_lock() {
        let mut game = having_bottle(|_| {});
        game.state = GameState::Lock(Duration::from_nanos(1));
        game.update(Duration::from_nanos(2));
        assert_eq!(game.state, GameState::Lock(Duration::from_nanos(3)));
        game.should_have_no_events();
    }

    #[test]
    fn update_lock_into_pattern() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_is_collision().return_once(|| true);
            bottle.expect_lock().return_once(|| Some(Vitamin::vitamins(PillShape::RED_BLUE)));
        });
        game.state = GameState::LOCK_NOW;
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::Pattern);
        game.should_have_events(&[GameEvent::Lock {
            player: 0,
            vitamins: Vitamin::vitamins(PillShape::RED_BLUE),
            hard_or_soft_dropped: false
        }]);
    }

    #[test]
    fn update_hard_drop_lock_into_pattern() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_is_collision().return_once(|| true);
            bottle.expect_lock().return_once(|| Some(Vitamin::vitamins(PillShape::RED_BLUE)));
        });
        game.hard_dropped = true;
        game.state = GameState::Lock(Duration::from_nanos(1));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::Pattern);
        game.should_have_events(&[GameEvent::Lock {
            player: 0,
            vitamins: Vitamin::vitamins(PillShape::RED_BLUE),
            hard_or_soft_dropped: true
        }])
    }

    #[test]
    fn update_lock_into_fall() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_is_collision().return_once(|| false);
        });
        game.state = GameState::LOCK_NOW;
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_FALL);
        game.should_have_no_events();
    }

    #[test]
    fn update_pattern_into_destroy() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_pattern().return_once(|| HashSet::from_iter([BottlePoint::new(1, 2)]));
        });
        game.state = GameState::Pattern;
        game.update(Duration::from_nanos(1));
        let points = HashSet::from_iter([BottlePoint::new(1, 2)]);
        assert_eq!(game.state, GameState::Destroy(points.clone()));
        game.should_have_events(&[GameEvent::Destroy(points)]);
    }

    #[test]
    fn update_destroy_into_drop_garbage() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_destroy()
                .with(eq(HashSet::from_iter([BottlePoint::new(1, 2)])))
                .return_once(|_| ());
        });

        game.state = GameState::Destroy(HashSet::from_iter([BottlePoint::new(1, 2)]));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_DROP_GARBAGE);
        game.should_have_no_events();
    }

    #[test]
    fn update_drop_garbage_into_drop_garbage() {
        let mut game = having_bottle(|_| {});
        game.state = GameState::DropGarbage(Duration::from_nanos(2));
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::DropGarbage(Duration::from_nanos(3)));
        game.should_have_no_events();
    }

    #[test]
    fn update_drop_garbage_into_next_drop_garbage() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_step_down_garbage().return_once(|| true);
        });
        game.state = GameState::DropGarbage(GARBAGE_DROP_DURATION);
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_DROP_GARBAGE);
        game.should_have_events(&[GameEvent::DropGarbage])
    }

    #[test]
    fn update_drop_garbage_into_spawn() {
        let mut game = having_bottle(|bottle| {
            bottle.expect_step_down_garbage().return_once(|| false);
        });
        game.state = GameState::DropGarbage(GARBAGE_DROP_DURATION);
        game.update(Duration::from_nanos(1));
        assert_eq!(game.state, GameState::NEW_SPAWN);
        game.should_have_no_events();
    }

    fn having_bottle<F>(mut f: F) -> Game
        where
            F: FnMut(&mut MockBottle) {
        let mut bottle = MockBottle::new();
        f(&mut bottle);
        Game::from_bottle(0, 10, GameSpeed::Low, GameRandom::from_u64_seed(12345), bottle)
    }

    trait GameTestHarness {
        fn should_have_no_events(&self);
        fn should_have_events(&self, events: &[GameEvent]);
    }

    impl GameTestHarness for Game {
        fn should_have_no_events(&self) {
            assert!(self.events.is_empty());
        }

        fn should_have_events(&self, events: &[GameEvent]) {
            assert_eq!(self.events, events.to_vec());
        }
    }

}