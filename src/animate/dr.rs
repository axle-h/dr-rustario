use std::time::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DrAnimationType {
    Static,
    Linear {
        fps: u32,
    },
    YoYo {
        fps: u32,
    },
    LinearWithPause {
        fps: u32,
        pause_for: Duration,
        resume_from_frame: usize,
    },
}

impl DrAnimationType {
    pub const RETRO_THROW: Self = Self::LinearWithPause {
        fps: 10,
        pause_for: Duration::from_millis(200),
        resume_from_frame: 0,
    };

    pub const NES_SNES_VICTORY: Self = Self::Linear {
        fps: 4,
    };
    pub const N64_VICTORY: Self = Self::LinearWithPause {
        fps: 7,
        pause_for: Duration::from_millis(2000),
        resume_from_frame: 0,
    };

    pub const N64_GAME_OVER: Self = Self::LinearWithPause {
        fps: 7,
        pause_for: Duration::from_millis(2000),
        resume_from_frame: 18,
    };

    pub fn fps(&self) -> Option<u32> {
        match self {
            DrAnimationType::Static => None,
            DrAnimationType::Linear { fps } => Some(*fps),
            DrAnimationType::YoYo { fps } => Some(*fps),
            DrAnimationType::LinearWithPause { fps, .. } => Some(*fps)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DrAnimation {
    animation_type: DrAnimationType,
    duration: Duration,
    frame_duration: Duration,
    paused_for: Option<Duration>,
    frame: usize,
    invert: bool,
    iteration: usize,
    max_frame: usize,
}

impl DrAnimation {
    pub fn new(animation_type: DrAnimationType, max_frame: usize) -> Self {
        let frame_duration = animation_type.fps()
            .map(|fps| Duration::from_secs(1) / fps)
            .unwrap_or(Duration::ZERO);
        Self {
            animation_type,
            duration: Duration::ZERO,
            frame_duration,
            paused_for: None,
            frame: 0,
            invert: false,
            iteration: 0,
            max_frame,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        self.duration += delta;
        match self.animation_type {
            DrAnimationType::Static => {
                self.frame = 0;
                self.iteration = 0;
            }
            DrAnimationType::Linear { .. } => {
                self.next_linear(false);
            }
            DrAnimationType::YoYo { .. } => {
                self.next_linear(true);
            }
            DrAnimationType::LinearWithPause {
                pause_for,
                resume_from_frame,
                ..
            } => {
                if let Some(paused_for) = self.paused_for {
                    // maybe unpause
                    self.paused_for = paused_for.checked_sub(delta);
                    if self.paused_for == Some(Duration::ZERO) {
                        self.paused_for = None;
                    }
                    if self.paused_for.is_none() {
                        // unpause
                        self.duration = Duration::ZERO;
                        self.iteration += 1;
                        self.frame = resume_from_frame;
                    }
                } else {
                    self.register_frames();
                    if self.frame >= self.max_frame {
                        // pause
                        self.frame = self.max_frame - 1;
                        self.paused_for = Some(pause_for);
                    }
                }
            }
        }
    }

    fn register_frames(&mut self) {
        loop {
            if let Some(remainder) = self.duration.checked_sub(self.frame_duration) {
                self.duration = remainder;
                self.frame += 1;
            } else {
                break;
            }
        }
    }

    fn next_linear(&mut self, invert: bool) {
        self.register_frames();
        if self.frame >= self.max_frame {
            self.iteration += 1;
            self.frame %= self.max_frame;
            if invert {
                self.invert = !self.invert;
            }
        }
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

    pub fn iteration(&self) -> usize {
        self.iteration
    }
}
