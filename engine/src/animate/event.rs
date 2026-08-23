#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AnimationType {
    Spawn,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AnimationEvent {
    Finished {
        player: u32,
        animation: AnimationType,
    },
}
