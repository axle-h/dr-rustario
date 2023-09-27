use std::time::Duration;
use crate::animate::dr::{DrAnimation, DrAnimationType};

const VISIBLE_FOR: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct State {
    duration: Duration,
    dr: DrAnimation,
    is_complete: bool
}

impl State {
    fn new(dr_animation: DrAnimation) -> Self {
        Self { duration: Duration::ZERO, dr: dr_animation, is_complete: false }
    }

    pub fn dr_frame(&self) -> usize {
        self.dr.frame()
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }
}

#[derive(Debug, Clone)]
pub struct VictoryAnimation {
    dr_frames: usize,
    dr_type: DrAnimationType,
    state: Option<State>
}

impl VictoryAnimation {
    pub fn new(dr_frames: usize, dr_type: DrAnimationType) -> Self {
        Self { dr_frames, dr_type, state: None }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
            state.dr.update(delta);
            state.is_complete = state.dr.iteration() > 0 && state.duration > VISIBLE_FOR;
        }
    }

    pub fn victory(&mut self) {
        let dr = DrAnimation::new(self.dr_type, self.dr_frames);
        self.state = Some(State::new(dr));
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }

    pub fn dismiss(&mut self) {
        if let Some(state) = self.state.as_mut() {
            state.is_complete = true;
        }
    }
}