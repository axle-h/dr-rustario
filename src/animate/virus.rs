use std::time::Duration;

const FRAME_DURATION: Duration = Duration::from_millis(200);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VirusAnimationType {
    Linear,
    YoYo
}


#[derive(Clone, Debug)]
pub struct VirusAnimation {
    frame: usize,
    invert: bool,
    max_frame: usize,
    animation_type: VirusAnimationType,
    duration: Duration
}

impl VirusAnimation {
    pub fn new(max_frame: usize, animation_type: VirusAnimationType) -> Self {
        Self { frame: 0, invert: false, max_frame, duration: Duration::ZERO, animation_type }
    }

    pub fn update(&mut self, delta: Duration) {
        self.duration += delta;
        self.frame = (self.duration.as_millis() / FRAME_DURATION.as_millis()) as usize % self.max_frame;

        if self.animation_type == VirusAnimationType::YoYo {
            let iteration = self.duration.as_millis() / (FRAME_DURATION * self.max_frame as u32).as_millis();
            self.invert = iteration % 2 == 1;
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.duration = Duration::ZERO;
    }

    pub fn frame(&self) -> usize {
        if self.invert {
            self.max_frame - self.frame - 1
        } else {
            self.frame
        }
    }
}