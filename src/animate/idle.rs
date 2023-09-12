use std::time::Duration;

const FRAME_DURATION: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
pub struct IdleAnimation {
    duration: Duration,
    frame: usize,
    invert: bool,
    max_frame: usize,
}

impl IdleAnimation {
    pub fn new(max_frame: usize) -> Self {
        Self { duration: Duration::ZERO, frame: 0, invert: false, max_frame }
    }

    pub fn update(&mut self, delta: Duration) {
        self.duration += delta;
        self.frame = (self.duration.as_millis() / FRAME_DURATION.as_millis()) as usize % self.max_frame;
        let iteration = self.duration.as_millis() / (FRAME_DURATION * self.max_frame as u32).as_millis();
        self.invert = iteration % 2 == 1;
    }

    pub fn reset(&mut self) {
        self.duration = Duration::ZERO;
        self.frame = 0;
        self.invert = false;
    }

    pub fn frame(&self) -> usize {
        if self.invert {
            self.max_frame - self.frame - 1
        } else {
            self.frame
        }
    }
}