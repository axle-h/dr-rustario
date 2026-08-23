use crate::animate::frames::{FrameAnimation, FrameAnimationType};

/// A character beside the board (Dr. Mario) with a sprite strip for each thing it does.
#[derive(Clone, Copy, Debug)]
pub struct MascotMeta {
    pub idle_type: FrameAnimationType,
    pub idle_frames: usize,
    pub spawn_type: FrameAnimationType,
    pub spawn_frames: usize,
    pub victory_type: FrameAnimationType,
    pub victory_frames: usize,
    pub game_over_type: FrameAnimationType,
    pub game_over_frames: usize,
}

impl MascotMeta {
    pub fn idle(&self) -> FrameAnimation {
        FrameAnimation::new(self.idle_type, self.idle_frames)
    }

    pub fn spawn(&self) -> FrameAnimation {
        FrameAnimation::new(self.spawn_type, self.spawn_frames)
    }

    pub fn victory(&self) -> FrameAnimation {
        FrameAnimation::new(self.victory_type, self.victory_frames)
    }

    pub fn game_over(&self) -> FrameAnimation {
        FrameAnimation::new(self.game_over_type, self.game_over_frames)
    }
}
