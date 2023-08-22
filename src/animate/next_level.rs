use std::time::Duration;
use rand::thread_rng;
use rand::seq::SliceRandom;
use rand::prelude::ThreadRng;
use crate::game::event::ColoredBlock;
use crate::game::geometry::BottlePoint;

const VIRUS_POP_IN_DURATION: Duration = Duration::from_millis(1500);
const NEXT_VIRUS_DURATION: Duration = Duration::from_millis(100);
const WAIT_DR_FRAME_DURATION: Duration = Duration::from_millis(300);
const INTERSTITIAL_FRAME_DURATION: Duration = Duration::from_millis(300);

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    DisplayInterstitial(usize),
    PopInViruses(Vec<ColoredBlock>)
}

impl Action {
    pub fn maybe_display_viruses(&self) -> Option<&Vec<ColoredBlock>> {
        if let Self::PopInViruses(viruses) = self {
            Some(viruses)
        } else {
            None
        }
    }

    pub fn interstitial_frame(&self) -> Option<usize> {
        if let Self::DisplayInterstitial(frame) = self {
            Some(*frame)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    DisplayInterstitial(usize),
    PopInViruses(Duration),
}

#[derive(Clone, Debug)]
pub struct StateWrapper {
    viruses: Vec<ColoredBlock>,
    next_virus_duration: Duration,
    duration: Duration,
    state: State,
    dr_frame: usize,
    interstitial_frames: usize
}

impl StateWrapper {
    pub fn new(viruses: Vec<ColoredBlock>, display_interstitial: bool, interstitial_frames: usize) -> Self {
        Self {
            next_virus_duration: NEXT_VIRUS_DURATION.min(VIRUS_POP_IN_DURATION / viruses.len() as u32),
            viruses,
            state: if display_interstitial { State::DisplayInterstitial(0) } else { State::PopInViruses(Duration::ZERO) },
            dr_frame: 0,
            duration: Duration::ZERO,
            interstitial_frames
        }
    }

    pub fn action(&self) -> Action {
        match self.state {
            State::DisplayInterstitial(frame) => Action::DisplayInterstitial(frame),
            State::PopInViruses(duration) => {
                let count = ((duration.as_millis() / self.next_virus_duration.as_millis()) as usize)
                    .min(self.viruses.len());
                Action::PopInViruses(self.viruses.iter().take(count).copied().collect())
            }
        }
    }

    pub fn dr_frame(&self) -> usize {
        self.dr_frame
    }

    fn update(&mut self, delta: Duration) {
        self.duration += delta;
        self.state = match self.state {
            State::DisplayInterstitial(_) => State::DisplayInterstitial(
                (self.duration.as_millis() / INTERSTITIAL_FRAME_DURATION.as_millis()) as usize % self.interstitial_frames
            ),
            State::PopInViruses(duration) => State::PopInViruses(duration + delta)
        };
    }

    fn is_finished(&self) -> bool {
        if let State::PopInViruses(duration) = self.state {
            duration >= VIRUS_POP_IN_DURATION
        } else {
            false
        }
    }

    fn maybe_dismiss_interstitial(&mut self) -> bool {
        if matches!(self.state, State::DisplayInterstitial(_)) {
            self.state = State::PopInViruses(Duration::ZERO);
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub struct NextLevelAnimation {
    dr_frames: usize,
    interstitial_frames: usize,
    state: Option<StateWrapper>,
    rng: ThreadRng
}

impl NextLevelAnimation {
    pub fn new(dr_frames: usize, interstitial_frames: usize) -> Self {
        Self { dr_frames, state: None, rng: thread_rng(), interstitial_frames }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.update(delta);
            if state.is_finished() {
                self.state = None;
            } else {
                state.dr_frame = (state.duration.as_millis() / WAIT_DR_FRAME_DURATION.as_millis()) as usize % self.dr_frames;
            }
        }
    }

    pub fn state(&self) -> Option<&StateWrapper> {
        self.state.as_ref()
    }

    pub fn maybe_dismiss_interstitial(&mut self) -> bool {
        if let Some(state) = self.state.as_mut() {
            state.maybe_dismiss_interstitial()
        } else {
            false
        }
    }

    pub fn next_level(&mut self, viruses: &[ColoredBlock], display_interstitial: bool) {
        let mut viruses = viruses.to_vec();
        viruses.shuffle(&mut self.rng);
        self.state = Some(StateWrapper::new(viruses, display_interstitial, self.interstitial_frames));
    }
}