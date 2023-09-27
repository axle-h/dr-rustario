use std::time::Duration;
use crate::game::pill::VirusColor;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VirusAnimationType {
    Linear { fps: u32 },
    YoYo { fps: u32 }
}

impl VirusAnimationType {
    pub const LINEAR_STANDARD: Self = Self::Linear { fps: 3 };
    pub const YO_YO_STANDARD: Self = Self::YoYo { fps: 3 };

    fn is_yo_yo(&self) -> bool {
        matches!(self, &Self::YoYo { .. })
    }

    pub fn fps(&self) -> u32 {
        match self {
            VirusAnimationType::Linear { fps } => *fps,
            VirusAnimationType::YoYo { fps } => *fps
        }
    }

    pub fn frame_duration(&self) -> Duration {
        Duration::from_secs(1) / self.fps()
    }
}

#[derive(Clone, Debug)]
struct VirusAnimationState {
    duration: Duration,
    frame: usize,
    invert: bool,
    max_frame: usize
}

impl VirusAnimationState {
    fn new(max_frame: usize) -> Self {
        Self { duration: Duration::ZERO, frame: 0, invert: false, max_frame }
    }

    fn update(&mut self, delta: Duration, frame_duration: Duration, is_yo_yo: bool) {
        self.duration += delta;

        loop {
            if let Some(remainder) = self.duration.checked_sub(frame_duration) {
                self.duration = remainder;
                self.frame += 1;
            } else {
                break;
            }
        }

        if self.frame >= self.max_frame {
            self.frame %= self.max_frame;
            if is_yo_yo {
                self.invert = !self.invert;
            }
        }
    }

    fn frame(&self) -> usize {
        if self.invert {
            self.max_frame - self.frame - 1
        } else {
            self.frame
        }
    }

    fn reset(&mut self) {
        self.duration = Duration::ZERO;
        self.frame = 0;
        self.invert = false;
    }
}

#[derive(Clone, Debug)]
pub struct VirusAnimation {
    red: VirusAnimationState,
    blue: VirusAnimationState,
    yellow: VirusAnimationState,
    animation_type: VirusAnimationType,
    frame_duration: Duration
}

impl VirusAnimation {
    pub fn new(red_frames: usize, blue_frames: usize, yellow_frames: usize, animation_type: VirusAnimationType) -> Self {
        let frame_duration = animation_type.frame_duration();
        Self {
            red: VirusAnimationState::new(red_frames),
            blue: VirusAnimationState::new(blue_frames),
            yellow: VirusAnimationState::new(yellow_frames),
            animation_type,
            frame_duration
        }
    }

    pub fn update(&mut self, delta: Duration) {
        let is_yo_yo = self.animation_type.is_yo_yo();
        self.red.update(delta, self.frame_duration, is_yo_yo);
        self.blue.update(delta, self.frame_duration, is_yo_yo);
        self.yellow.update(delta, self.frame_duration, is_yo_yo);
    }

    pub fn reset(&mut self) {
        self.red.reset();
        self.blue.reset();
        self.yellow.reset();
    }

    pub fn frame(&self, color: VirusColor) -> usize {
        match color {
            VirusColor::Yellow => self.yellow.frame(),
            VirusColor::Blue => self.blue.frame(),
            VirusColor::Red => self.red.frame(),
        }
    }
}