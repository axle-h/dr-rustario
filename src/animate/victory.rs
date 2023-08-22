use std::time::Duration;

const VICTORY_DR_FRAME_DURATION: Duration = Duration::from_millis(250);
const VISIBLE_FOR: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct State {
    duration: Duration,
    dr_frame: usize,
    is_complete: bool
}

impl State {
    fn new() -> Self {
        Self { duration: Duration::ZERO, dr_frame: 0, is_complete: false }
    }

    pub fn dr_frame(&self) -> usize {
        self.dr_frame
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }
}

#[derive(Debug, Clone)]
pub struct VictoryAnimation {
    dr_victory_frames: usize,
    state: Option<State>
}

impl VictoryAnimation {
    pub fn new(dr_victory_frames: usize) -> Self {
        Self { dr_victory_frames, state: None }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
            state.dr_frame = (state.duration.as_millis() / VICTORY_DR_FRAME_DURATION.as_millis()) as usize % self.dr_victory_frames;
            state.is_complete = state.is_complete || state.duration >= VICTORY_DR_FRAME_DURATION * self.dr_victory_frames as u32 + VISIBLE_FOR;
        }
    }

    pub fn victory(&mut self) {
        self.state = Some(State::new());
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