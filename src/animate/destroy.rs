use std::collections::HashSet;
use std::time::Duration;
use crate::game::event::ColoredBlock;
use crate::game::geometry::BottlePoint;

const VITAMIN_POP_DURATION: Duration = Duration::from_millis(300);

#[derive(Clone, Debug)]
pub struct State {
    blocks: Vec<ColoredBlock>,
    frame: usize,
    duration: Duration
}

impl State {
    pub fn frame(&self) -> usize {
        self.frame
    }
    pub fn blocks(&self) -> Vec<ColoredBlock> {
        self.blocks.clone()
    }
}

#[derive(Clone, Debug)]
pub struct DestroyAnimation {
    max_frames: usize,
    frame_duration: Duration,
    state: Option<State>
}

impl DestroyAnimation {
    pub fn new(max_frames: usize) -> Self {
        let frame_duration = VITAMIN_POP_DURATION / max_frames as u32;
        Self { max_frames, frame_duration, state: None }
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
            finished = state.frame == self.max_frames;
        }
        if finished {
            self.state = None;
        }
    }

    pub fn reset(&mut self) {
        self.state = None;
    }

    pub fn add(&mut self, blocks: Vec<ColoredBlock>) {
        self.state = Some(State { blocks, frame: 0, duration: Duration::ZERO })
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }
}