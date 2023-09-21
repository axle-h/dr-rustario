use std::time::Duration;
use crate::animate::dr::{DrAnimation, DrAnimationType};

#[derive(Debug, Clone)]
pub struct IdleAnimation {
    dr: DrAnimation
}

impl IdleAnimation {
    pub fn new(max_frame: usize, dr_type: DrAnimationType) -> Self {
        Self { dr: DrAnimation::new(dr_type, max_frame) }
    }

    pub fn update(&mut self, delta: Duration) {
        self.dr.update(delta);
    }

    pub fn reset(&mut self) {
        self.dr.reset();
    }

    pub fn frame(&self) -> usize {
        self.dr.frame()
    }
}