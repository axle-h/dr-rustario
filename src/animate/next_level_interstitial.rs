use std::time::Duration;
use crate::animate::dr::{DrAnimation, DrAnimationType};

const INTERSTITIAL_ITERATION_DURATION: Duration = Duration::from_millis(600);

#[derive(Clone, Copy, Debug)]
pub struct State {
    dr: DrAnimation,
    interstitial_frame: usize
}

impl State {
    fn new(dr: DrAnimation) -> Self {
        Self { dr, interstitial_frame: 0 }
    }

    pub fn interstitial_frame(&self) -> usize {
        self.interstitial_frame
    }

    pub fn dr_frame(&self) -> usize {
        self.dr.frame()
    }

}

#[derive(Clone, Debug)]
pub struct NextLevelInterstitialAnimation {
    state: Option<State>,
    dr_type: DrAnimationType,
    dr_frames: usize,
    interstitial_frames: usize
}

impl NextLevelInterstitialAnimation {
    pub fn new(dr_type: DrAnimationType, dr_frames: usize, interstitial_frames: usize) -> Self {
        Self { state: None, dr_type, dr_frames, interstitial_frames }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.dr.update(delta);

            let interstitial_frame_duration = INTERSTITIAL_ITERATION_DURATION / self.interstitial_frames as u32;
            state.interstitial_frame = (state.dr.duration().as_millis() / interstitial_frame_duration.as_millis()) as usize % self.interstitial_frames;
        }
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }

    pub fn dismiss(&mut self) -> bool {
        if self.state.is_some() {
            self.state = None;
            true
        } else {
            false
        }
    }

    pub fn display(&mut self) {
        let dr = DrAnimation::new(self.dr_type, self.dr_frames);
        self.state = Some(State::new(dr));
    }
}