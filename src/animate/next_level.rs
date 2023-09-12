use std::time::Duration;
use rand::thread_rng;
use rand::seq::SliceRandom;
use rand::prelude::ThreadRng;
use crate::game::event::ColoredBlock;
use crate::game::geometry::BottlePoint;

const VIRUS_POP_IN_DURATION: Duration = Duration::from_millis(1500);
const NEXT_VIRUS_DURATION: Duration = Duration::from_millis(100);

#[derive(Clone, Debug)]
pub struct State {
    viruses: Vec<ColoredBlock>,
    duration: Duration,
}

impl State {
    fn new(viruses: Vec<ColoredBlock>) -> Self {
        Self { viruses, duration: Duration::ZERO }
    }

    pub fn display_viruses(&self) -> Vec<ColoredBlock> {
        let next_virus_duration = NEXT_VIRUS_DURATION.min(VIRUS_POP_IN_DURATION / self.viruses.len() as u32);
        let count = ((self.duration.as_millis() / next_virus_duration.as_millis()) as usize)
            .min(self.viruses.len());
        self.viruses.iter().take(count).copied().collect()
    }
}

#[derive(Clone, Debug)]
pub struct NextLevelAnimation {
    state: Option<State>,
    rng: ThreadRng
}

impl NextLevelAnimation {
    pub fn new() -> Self {
        Self { state: None, rng: thread_rng() }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
            if state.duration >= VIRUS_POP_IN_DURATION {
                self.state = None;
            }
        }
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }

    pub fn next_level(&mut self, viruses: &[ColoredBlock]) {
        let mut viruses = viruses.to_vec();
        viruses.shuffle(&mut self.rng);
        self.state = Some(State::new(viruses));
    }
}