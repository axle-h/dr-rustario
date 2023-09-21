use std::time::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VirusAnimationType {
    Linear { fps: u32 },
    YoYo { fps: u32 }
}

impl VirusAnimationType {
    pub const LINEAR_STANDARD: Self = Self::Linear { fps: 3 };
    pub const YO_YO_STANDARD: Self = Self::Linear { fps: 3 };

    fn is_yo_yo(&self) -> bool {
        matches!(self, &Self::YoYo { .. })
    }

    pub fn fps(&self) -> u32 {
        match self {
            VirusAnimationType::Linear { fps } => *fps,
            VirusAnimationType::YoYo { fps } => *fps
        }
    }
}

#[derive(Clone, Debug)]
pub struct VirusAnimation {
    frame: usize,
    invert: bool,
    max_frame: usize,
    animation_type: VirusAnimationType,
    duration: Duration,
    frame_duration: Duration
}

impl VirusAnimation {
    pub fn new(max_frame: usize, animation_type: VirusAnimationType) -> Self {
        let frame_duration = Duration::from_secs(1) / animation_type.fps();
        Self { frame: 0, invert: false, max_frame, duration: Duration::ZERO, animation_type, frame_duration }
    }

    pub fn update(&mut self, delta: Duration) {
        self.duration += delta;
        self.frame = (self.duration.as_millis() / self.frame_duration.as_millis()) as usize % self.max_frame;

        if self.animation_type.is_yo_yo() {
            let iteration = self.duration.as_millis() / (self.frame_duration * self.max_frame as u32).as_millis();
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