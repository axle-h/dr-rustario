use std::collections::HashSet;
use std::time::Duration;
use crate::game::event::ColoredBlock;
use crate::game::geometry::BottlePoint;

const POP_DURATION: Duration = Duration::from_millis(300);

#[derive(Clone, Debug)]
pub struct State {
    blocks: Vec<ColoredBlock>,
    vitamin_frame: usize,
    virus_frame: usize,
    duration: Duration
}

impl State {
    pub fn vitamin_frame(&self) -> usize {
        self.vitamin_frame
    }
    pub fn virus_frame(&self) -> usize {
        self.virus_frame
    }
    pub fn blocks(&self) -> Vec<ColoredBlock> {
        self.blocks.clone()
    }
}

#[derive(Clone, Debug)]
pub struct DestroyAnimation {
    vitamin_frames: usize,
    vitamin_duration: Duration,
    virus_frames: usize,
    virus_duration: Duration,
    state: Option<State>
}

impl DestroyAnimation {
    pub fn new(vitamin_frames: usize, virus_frames: usize) -> Self {
        assert!(vitamin_frames > 0 && virus_frames > 0);
        let vitamin_duration = POP_DURATION / vitamin_frames as u32;
        let virus_duration = POP_DURATION / virus_frames as u32;
        Self { vitamin_frames, vitamin_duration, virus_frames, virus_duration, state: None }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;

            let duration = state.duration.as_millis();
            state.vitamin_frame = (duration / self.vitamin_duration.as_millis()) as usize % self.vitamin_frames;
            state.virus_frame = (duration / self.virus_duration.as_millis()) as usize % self.virus_frames;
            if state.duration >= POP_DURATION {
                self.state = None;
            }
        }
    }

    pub fn reset(&mut self) {
        self.state = None;
    }

    pub fn add(&mut self, blocks: Vec<ColoredBlock>) {
        self.state = Some(State { blocks, vitamin_frame: 0, virus_frame: 0, duration: Duration::ZERO })
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }
}