use std::collections::HashSet;
use std::time::Duration;
use crate::game::geometry::BottlePoint;
use crate::game::pill::Vitamins;

const VITAMIN_LOCK_DURATION: Duration = Duration::from_millis(100);
const FRAMES: u32 = 1;
const MAX_OFFSET: f64 = 0.1;

#[derive(Clone, Debug)]
pub struct State {
    vitamins: HashSet<BottlePoint>,
    duration: Duration,
    frame: u32
}

impl State {
    fn new(vitamins: Vitamins) -> Self {
        Self {
            vitamins: HashSet::from_iter(vitamins.map(|v| v.position())),
            duration: Duration::ZERO,
            frame: 0
        }
    }

    pub fn animates(&self, point: BottlePoint) -> bool {
        self.vitamins.contains(&point)
    }

    /// offset the vitamins by specified percent of a block
    pub fn offset_y(&self) -> f64 {
        (self.frame + 1) as f64 * (MAX_OFFSET / FRAMES as f64)
    }
}

impl Default for State {
    fn default() -> Self {
        Self { vitamins: HashSet::new(), duration: Duration::ZERO, frame: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct LockAnimation {
    state: Option<State>,
    frame_duration: Duration,

}

impl LockAnimation {
    pub fn new() -> Self {
        Self { state: None, frame_duration: VITAMIN_LOCK_DURATION / FRAMES }
    }

    pub fn update(&mut self, delta: Duration) {
        let mut finished = false;
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
            if state.duration < self.frame_duration {
                return;
            }
            state.duration = Duration::ZERO;
            state.frame += 1;
            finished = state.frame == FRAMES;
        }
        if finished {
            self.state = None;
        }
    }

    pub fn reset(&mut self) {
        self.state = None;
    }

    pub fn lock(&mut self, vitamins: Vitamins) {
        self.state = Some(State::new(vitamins));
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }
}