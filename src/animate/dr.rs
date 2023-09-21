use std::time::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DrAnimationType {
    Static,
    Linear { duration: Duration },
    YoYo { duration: Duration },
    LinearWithPause  { duration: Duration, pause_for: Duration, resume_from_frame: usize }
}

impl DrAnimationType {
    pub const RETRO_THROW: Self = Self::LinearWithPause {
        duration: Duration::from_millis(100),
        pause_for: Duration::from_millis(200),
        resume_from_frame: 0
    };

    pub const NES_SNES_VICTORY: Self = Self::Linear { duration: Duration::from_millis(250) };
    pub const N64_VICTORY: Self = Self::LinearWithPause {
        duration: Duration::from_millis(150),
        pause_for: Duration::from_millis(2000),
        resume_from_frame: 0
    };

    pub const N64_GAME_OVER: Self = Self::LinearWithPause {
        duration: Duration::from_millis(150),
        pause_for: Duration::from_millis(2000),
        resume_from_frame: 18
    };
}

#[derive(Clone, Copy, Debug)]
pub struct DrAnimation {
    animation_type: DrAnimationType,
    duration: Duration,
    paused_for: Option<Duration>,
    frame: usize,
    invert: bool,
    iteration: usize,
    max_frame: usize
}

impl DrAnimation {
    pub fn new(animation_type: DrAnimationType, max_frame: usize) -> Self {
        Self {
            animation_type,
            duration: Duration::ZERO,
            paused_for: None,
            frame: 0,
            invert: false,
            iteration: 0,
            max_frame
        }
    }

    pub fn update(&mut self, delta: Duration) {
        self.duration += delta;
        match self.animation_type {
            DrAnimationType::Static => {
                self.frame = 0;
                self.iteration = 0;
            }
            DrAnimationType::Linear { duration: frame_duration } => {
                self.next_linear(frame_duration, false);
            }
            DrAnimationType::YoYo { duration: frame_duration } => {
                self.next_linear(frame_duration, true);
            }
            DrAnimationType::LinearWithPause { duration: frame_duration, pause_for, resume_from_frame } => {
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
                    self.register_frames(frame_duration);
                    if self.frame >= self.max_frame {
                        // pause
                        self.frame = self.max_frame - 1;
                        self.paused_for = Some(pause_for);
                    }
                }
            }
        }
    }

    fn register_frames(&mut self, frame_duration: Duration) {
        loop {
            if let Some(remainder) = self.duration.checked_sub(frame_duration) {
                self.duration = remainder;
                self.frame += 1;
            } else {
                break;
            }
        }
    }

    fn next_linear(&mut self, frame_duration: Duration, invert: bool) {
        self.register_frames(frame_duration);
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

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn iteration(&self) -> usize {
        self.iteration
    }
}

