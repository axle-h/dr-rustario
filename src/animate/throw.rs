use crate::animate::dr::{DrAnimation, DrAnimationType};

use crate::game::pill::PillShape;
use sdl2::rect::Point;
use std::f64::consts::PI;
use std::time::Duration;

const ARC_DURATION: f64 = 0.5; // secs
const ARC_HEIGHT_BLOCKS: f64 = 4.5;

#[derive(Clone, Debug)]
pub struct State {
    arc: LinearThrowArc,
    shape: PillShape,
    duration: f64,
    is_hold: bool,
    dr: DrAnimation,
}

impl State {
    fn new(arc: LinearThrowArc, shape: PillShape, is_hold: bool, dr: DrAnimation) -> Self {
        Self {
            shape,
            duration: 0.0,
            arc,
            dr,
            is_hold,
        }
    }

    pub fn throw_position(&self) -> Point {
        let x = self.arc.distance(self.duration);
        let y = self.arc.height(x);
        Point::new(x.round() as i32, y.round() as i32)
    }

    pub fn shape(&self) -> PillShape {
        self.shape
    }

    pub fn dr_throw_frame(&self) -> usize {
        self.dr.frame()
    }

    pub fn peek_offset(&self) -> Option<f64> {
        if self.is_hold {
            None
        } else {
            Some(self.duration / ARC_DURATION)
        }
    }

    pub fn pill_rotate_angle_degrees(&self) -> f64 {
        360.0 * self.duration / ARC_DURATION
    }
}

#[derive(Clone, Debug)]
pub struct ThrowAnimation {
    state: Option<State>,
    arc: LinearThrowArc,
    dr_frames: usize,
    dr_type: DrAnimationType,
}

impl ThrowAnimation {
    pub fn new(
        start: Point,
        end: Point,
        block_size: u32,
        dr_frames: usize,
        dr_type: DrAnimationType,
    ) -> Self {
        Self {
            state: None,
            arc: LinearThrowArc::new(start, end, block_size),
            dr_frames,
            dr_type,
        }
    }

    pub fn update(&mut self, delta: Duration) -> bool {
        let mut finished = false;
        if let Some(animation) = self.state.as_mut() {
            animation.duration += delta.as_secs_f64();
            if animation.duration > ARC_DURATION {
                finished = true
            } else if animation.dr.iteration() == 0 {
                animation.dr.update(delta);
            }
        }
        if finished {
            self.state = None;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.state = None;
    }

    pub fn throw(&mut self, shape: PillShape, is_hold: bool) {
        let dr = DrAnimation::new(self.dr_type, self.dr_frames);
        self.state = Some(State::new(self.arc, shape, is_hold, dr));
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }
}

/// this is a linear function plus half a sin wave (i.e. 0 -> pi)
#[derive(Clone, Copy, Debug)]
struct LinearThrowArc {
    x_start: f64,
    x_end: f64,
    magnitude: f64,
    m: f64,
    c: f64,
}

impl LinearThrowArc {
    fn new(start: Point, end: Point, block_size: u32) -> Self {
        let m = (end.y() as f64 - start.y() as f64) / (end.x() as f64 - start.x() as f64);
        let c = start.y() as f64 - m * start.x() as f64;
        Self {
            x_start: start.x() as f64,
            x_end: end.x() as f64,
            magnitude: block_size as f64 * ARC_HEIGHT_BLOCKS,
            m,
            c,
        }
    }

    fn distance(&self, duration: f64) -> f64 {
        self.x_start + (self.x_end - self.x_start) * duration / ARC_DURATION
    }

    fn height(&self, x: f64) -> f64 {
        let linear = self.m * x + self.c;
        let wave = self.magnitude * (PI * (self.x_end - x) / (self.x_end - self.x_start)).sin();
        // take the wave function away since a lower number is actually higher
        linear - wave
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_height() {
        let f = LinearThrowArc::new(Point::new(190, 62), Point::new(120, 72), 7);
        assert_eq!(f.height(155.0), 35.5);
    }

    #[test]
    fn arc_distance() {
        let f = LinearThrowArc::new(Point::new(190, 62), Point::new(120, 72), 7);
        assert_eq!(f.distance(ARC_DURATION / 2.0), 155.0);
    }
}
