use std::time::Duration;

const WAIT_DR_FRAME_DURATION: Duration = Duration::from_millis(300);
const INTERSTITIAL_ITERATION_DURATION: Duration = Duration::from_millis(600);

#[derive(Clone, Copy, Debug)]
pub struct State {
    duration: Duration,
    dr_frames: usize,
    interstitial_frames: usize
}

impl State {
    fn new(dr_frames: usize, interstitial_frames: usize) -> Self {
        Self { duration: Duration::ZERO, dr_frames, interstitial_frames }
    }

    pub fn interstitial_frame(&self) -> usize {
        let interstitial_frame_duration = INTERSTITIAL_ITERATION_DURATION / self.interstitial_frames as u32;
        (self.duration.as_millis() / interstitial_frame_duration.as_millis()) as usize % self.interstitial_frames
    }

    pub fn dr_frame(&self) -> usize {
        (self.duration.as_millis() / WAIT_DR_FRAME_DURATION.as_millis()) as usize % self.dr_frames
    }

}

#[derive(Clone, Debug)]
pub struct NextLevelInterstitialAnimation {
    state: Option<State>,
    dr_frames: usize,
    interstitial_frames: usize
}

impl NextLevelInterstitialAnimation {
    pub fn new(dr_frames: usize, interstitial_frames: usize) -> Self {
        Self { state: None, dr_frames, interstitial_frames }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
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
        self.state = Some(State::new(self.dr_frames, self.interstitial_frames));
    }
}