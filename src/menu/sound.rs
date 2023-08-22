use std::rc::Rc;
use crate::config::AudioConfig;
use crate::theme::sound::{LoadSound, Playable, StructuredMusic};
use sdl2::mixer::{Chunk, Music};

const CHIME: &[u8] = include_bytes!("chime.ogg");
const TITLE_INTRO: &'static [u8] = include_bytes!("title-intro.ogg");
const TITLE_REPEAT: &'static [u8] = include_bytes!("title-repeat.ogg");
const MENU_INTRO: &'static [u8] = include_bytes!("menu-intro.ogg");
const MENU_REPEAT: &'static [u8] = include_bytes!("menu-repeat.ogg");
const HIGH_SCORE_INTRO: &'static [u8] = include_bytes!("high-score-intro.ogg");
const HIGH_SCORE_REPEAT: &'static [u8] = include_bytes!("high-score-repeat.ogg");

pub struct MenuSound {
    chime: Chunk,
    menu_music: Rc<StructuredMusic>,
    title_music: Rc<StructuredMusic>,
    high_score_music: Rc<StructuredMusic>,
}

impl MenuSound {
    pub fn new(config: AudioConfig) -> Result<Self, String> {
        Ok(Self {
            chime: config.load_chunk(CHIME)?,
            menu_music: StructuredMusic::new(MENU_INTRO, MENU_REPEAT)?.into_rc(),
            title_music: StructuredMusic::new(TITLE_INTRO, TITLE_REPEAT)?.into_rc(),
            high_score_music: StructuredMusic::new(HIGH_SCORE_INTRO, HIGH_SCORE_REPEAT)?.into_rc(),
        })
    }

    pub fn play_chime(&self) -> Result<(), String> {
        self.chime.play()
    }

    pub fn play_title_music(&self) -> Result<(), String> {
        StructuredMusic::play(&self.title_music)
    }

    pub fn play_menu_music(&self) -> Result<(), String> {
        StructuredMusic::play(&self.menu_music)
    }

    pub fn play_high_score_music(&self) -> Result<(), String> {
        StructuredMusic::play(&self.high_score_music)
    }
}
