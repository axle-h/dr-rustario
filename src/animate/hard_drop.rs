use std::time::Duration;
use crate::game::pill::Vitamins;

const FRAME_DURATION: f64 = 0.004; // 4 millis
const MAX_ALPHA: u8 = 100;
const MAX_TRAIL_FRAMES: u32 = 5;
const STEP_SIZE: f64 = 0.2;

#[derive(Clone, Debug)]
pub struct State {
    vitamins: Vitamins,
    duration: f64,
    frame: u32,
    max_frames: u32,
}

impl State {
    pub fn new(vitamins: Vitamins, dropped_rows: u32) -> Self {
        let max_frames = dropped_rows as f64 / STEP_SIZE;
        Self {
            vitamins,
            max_frames: max_frames.round() as u32,
            duration: 0.0,
            frame: 0
        }
    }

    pub fn frames(&self) -> Vec<HardDropAnimationFrame> {
        let mut result = vec![];
        let trail_frames = self.frame.min(MAX_TRAIL_FRAMES);
        for j in 1..=trail_frames {
            let alpha_mod = MAX_ALPHA - (MAX_ALPHA as f64 * j as f64 / trail_frames as f64).round() as u8;
            let offset_y = -1.0 * STEP_SIZE * j as f64;
            result.push(HardDropAnimationFrame::new(offset_y, alpha_mod));
        }

        // fall
        for j in 1..=self.frame {
            let alpha_mod = MAX_ALPHA - (MAX_ALPHA as f64 * j as f64 / self.max_frames as f64).round() as u8;
            let offset_y = STEP_SIZE * j as f64;
            result.push(HardDropAnimationFrame::new(offset_y, alpha_mod));
        }

        result
    }

    pub fn vitamins(&self) -> Vitamins {
        self.vitamins
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HardDropAnimationFrame {
    pub offset_y: f64,
    pub alpha_mod: u8
}

impl HardDropAnimationFrame {
    pub fn new(offset_y: f64, alpha_mod: u8) -> Self {
        Self { offset_y, alpha_mod }
    }
}

#[derive(Debug, Clone)]
pub struct HardDropAnimation {
    state: Option<State>
}

impl HardDropAnimation {
    pub fn new() -> Self {
        Self { state: None }
    }

    pub fn update(&mut self, delta: Duration) {
        let mut finished = false;
        if let Some(state) = self.state.as_mut() {
            state.duration += delta.as_secs_f64();
            if state.duration < FRAME_DURATION {
                return;
            }

            let frame_delta = state.duration / FRAME_DURATION;
            state.duration = 0.0;
            state.frame += frame_delta.round() as u32;
            finished = state.frame >= state.max_frames;
        }
        if finished {
            self.state = None;
        }
    }

    pub fn reset(&mut self) {
        self.state = None;
    }

    pub fn hard_drop(&mut self, vitamins: Vitamins, dropped_rows: u32) {
        if dropped_rows > 0 {
            self.state = Some(State::new(vitamins, dropped_rows));
        }
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }
}