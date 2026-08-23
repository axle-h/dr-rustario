use crate::animate::frames::FrameAnimation;
use crate::animate::mascot::MascotMeta;
use std::time::Duration;

const INTERSTITIAL_ITERATION_DURATION: Duration = Duration::from_millis(600);

/// A "stage clear" card shown between stages until the player dismisses it.
#[derive(Clone, Copy, Debug)]
pub struct State {
    duration: Duration,
    mascot: Option<FrameAnimation>,
    interstitial_frame: usize,
}

impl State {
    pub fn interstitial_frame(&self) -> usize {
        self.interstitial_frame
    }

    pub fn mascot_frame(&self) -> Option<usize> {
        self.mascot.map(|m| m.frame())
    }
}

#[derive(Clone, Debug)]
pub struct InterstitialAnimation {
    state: Option<State>,
    mascot: Option<MascotMeta>,
    frames: usize,
}

impl InterstitialAnimation {
    pub fn new(frames: usize, mascot: Option<MascotMeta>) -> Self {
        Self {
            state: None,
            mascot,
            frames: frames.max(1),
        }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
            if let Some(mascot) = state.mascot.as_mut() {
                mascot.update(delta);
            }
            let frame_duration = INTERSTITIAL_ITERATION_DURATION / self.frames as u32;
            state.interstitial_frame =
                (state.duration.as_millis() / frame_duration.as_millis().max(1)) as usize
                    % self.frames;
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
        self.state = Some(State {
            duration: Duration::ZERO,
            mascot: self.mascot.map(|m| m.victory()),
            interstitial_frame: 0,
        });
    }
}
