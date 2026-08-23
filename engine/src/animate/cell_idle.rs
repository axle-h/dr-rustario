use crate::animate::frames::{FrameAnimation, FrameAnimationType};
use crate::game::CellId;
use std::collections::HashMap;
use std::time::Duration;

/// Cells that animate while sitting on the board (Dr. Mario's wriggling viruses).
#[derive(Clone, Debug)]
pub struct CellIdleAnimation {
    cells: HashMap<CellId, FrameAnimation>,
}

impl CellIdleAnimation {
    pub fn new(animation_type: FrameAnimationType, frames: &[(CellId, usize)]) -> Self {
        Self {
            cells: frames
                .iter()
                .map(|(id, frames)| (*id, FrameAnimation::new(animation_type, *frames)))
                .collect(),
        }
    }

    pub fn update(&mut self, delta: Duration) {
        for animation in self.cells.values_mut() {
            animation.update(delta);
        }
    }

    pub fn reset(&mut self) {
        for animation in self.cells.values_mut() {
            animation.reset();
        }
    }

    /// the current frame, or `None` if this cell does not animate
    pub fn frame(&self, id: CellId) -> Option<usize> {
        self.cells.get(&id).map(|a| a.frame())
    }
}
