use std::time::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DrAnimationType {
    Static,
    Linear { duration: Duration },
    LinearWithPause  { duration: Duration, pause_for: usize, resume_from_frame: usize }
}

impl DrAnimationType {
    pub const NES_SNES_VICTORY: Self = Self::Linear { duration: Duration::from_millis(500) };
    pub const N64_VICTORY: Self = Self::LinearWithPause {
        duration: Duration::from_millis(2000),
        pause_for: 1,
        resume_from_frame: 0
    };

    pub const N64_GAME_OVER: Self = Self::LinearWithPause {
        duration: Duration::from_millis(2000),
        pause_for: 1,
        resume_from_frame: 18
    };
}

#[derive(Clone, Copy, Debug)]
pub struct DrAnimation {
    animation_type: DrAnimationType,
    duration: Duration,
    paused_for: Option<Duration>,
    frame: usize,
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
            iteration: 0,
            max_frame
        }
    }

    pub fn update(&mut self, delta: Duration) {
        self.duration += delta;

        let duration = self.duration.as_millis();
        match self.animation_type {
            DrAnimationType::Linear { duration: total_duration } => {
                let frame_duration = total_duration / self.max_frame as u32;
                self.frame = (duration / frame_duration.as_millis()) as usize % self.max_frame;
                self.iteration = (duration / total_duration.as_millis()) as usize;
            }
            DrAnimationType::LinearWithPause { duration: total_duration, pause_for, resume_from_frame } => {
                let frame_duration = total_duration / self.max_frame as u32;
                let next_frame = (duration / frame_duration.as_millis()) as usize;

                let min_frame = if self.iteration > 0 { resume_from_frame } else { 0 };
                let frames_per_iteration = self.max_frame - min_frame;
                let next_iteration = next_frame / frames_per_iteration;

                if let Some(paused_for) = self.paused_for {
                    // maybe unpause
                    self.paused_for = paused_for.checked_sub(delta);
                    if self.paused_for == Some(Duration::ZERO) {
                        self.paused_for = None;
                    }
                } else if next_iteration > self.iteration {
                    // pause
                    self.paused_for = Some(total_duration * pause_for as u32);
                } else if self.paused_for.is_none() {
                    self.frame = min_frame + next_frame % frames_per_iteration;
                }

                self.iteration = next_iteration;
            }
            DrAnimationType::Static => {
                self.frame = 0;
                self.iteration = 0;
            }
        }
    }

    pub fn reset(&mut self) {
        self.duration = Duration::ZERO;
        self.frame = 0;
    }


    pub fn frame(&self) -> usize {
        self.frame
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn iteration(&self) -> usize {
        self.iteration
    }
}

