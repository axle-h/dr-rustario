use std::time::Duration;

const FRAME_DURATION: Duration = Duration::from_millis(200);

#[derive(Clone, Debug)]
pub struct VirusAnimation {
    frame: usize,
    max_frame: usize,
    duration: Duration
}

impl VirusAnimation {
    pub fn new(max_frame: usize) -> Self {
        Self { frame: 0, max_frame, duration: Duration::ZERO }
    }

    pub fn update(&mut self, delta: Duration) {
        self.duration += delta;
        if self.duration > FRAME_DURATION {
            self.duration = Duration::ZERO;
            self.frame = (self.frame + 1) % self.max_frame;
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.duration = Duration::ZERO;
    }

    pub fn frame(&self) -> usize {
        self.frame
    }
}