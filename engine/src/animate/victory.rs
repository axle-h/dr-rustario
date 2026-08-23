use crate::animate::frames::FrameAnimation;
use crate::animate::mascot::MascotMeta;
use std::time::Duration;

const VISIBLE_FOR: Duration = Duration::from_secs(10);

// the victory cannot be dismissed until it has been visible for at least this long, so keys
// still held from the end of the match don't skip it before the winner can be seen
const MIN_VISIBLE_FOR: Duration = Duration::from_secs(3);

#[derive(Debug, Clone)]
pub struct State {
    duration: Duration,
    mascot: Option<FrameAnimation>,
    is_complete: bool,
}

impl State {
    pub fn mascot_frame(&self) -> Option<usize> {
        self.mascot.map(|m| m.frame())
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }
}

#[derive(Debug, Clone)]
pub struct VictoryAnimation {
    mascot: Option<MascotMeta>,
    state: Option<State>,
}

impl VictoryAnimation {
    pub fn new(mascot: Option<MascotMeta>) -> Self {
        Self {
            mascot,
            state: None,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
            let mascot_done = match state.mascot.as_mut() {
                Some(mascot) => {
                    mascot.update(delta);
                    mascot.iteration() > 0
                }
                None => true,
            };
            state.is_complete = mascot_done && state.duration > VISIBLE_FOR;
        }
    }

    pub fn victory(&mut self) {
        self.state = Some(State {
            duration: Duration::ZERO,
            mascot: self.mascot.map(|m| m.victory()),
            is_complete: false,
        });
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }

    pub fn dismiss(&mut self) {
        if let Some(state) = self.state.as_mut() {
            if state.duration >= MIN_VISIBLE_FOR {
                state.is_complete = true;
            }
        }
    }
}
