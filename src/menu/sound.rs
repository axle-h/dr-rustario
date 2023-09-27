use std::rc::Rc;
use crate::config::AudioConfig;
use crate::theme::sound::{LoadSound, Playable, StructuredMusic};
use sdl2::mixer::{Chunk, Music};


// const CHIME: &[u8] = include_bytes!("retro/chime.ogg");
// const TITLE_INTRO: &'static [u8] = include_bytes!("retro/title-intro.ogg");
// const TITLE_REPEAT: &'static [u8] = include_bytes!("retro/title-repeat.ogg");
// const MENU_INTRO: &'static [u8] = include_bytes!("retro/menu-intro.ogg");
// const MENU_REPEAT: &'static [u8] = include_bytes!("retro/menu-repeat.ogg");
// const HIGH_SCORE_INTRO: &'static [u8] = include_bytes!("retro/high-score-intro.ogg");
// const HIGH_SCORE_REPEAT: &'static [u8] = include_bytes!("retro/high-score-repeat.ogg");

const CHIME: &[u8] = include_bytes!("modern/chime.ogg");
const SELECT: &[u8] = include_bytes!("modern/select.ogg");
const TITLE: &'static [u8] = include_bytes!("modern/title.ogg");
const MENU: &'static [u8] = include_bytes!("modern/menu.ogg");
const HIGH_SCORE_INTRO: &'static [u8] = include_bytes!("modern/high-score-intro.ogg");
const HIGH_SCORE_REPEAT: &'static [u8] = include_bytes!("modern/high-score-repeat.ogg");

pub struct MenuSound {
    chime: Chunk,
    select: Chunk,
    menu_music: Rc<StructuredMusic>,
    title_music: Rc<StructuredMusic>,
    high_score_music: Rc<StructuredMusic>,
}

impl MenuSound {
    pub fn new(config: AudioConfig) -> Result<Self, String> {
        Ok(Self {
            chime: config.load_chunk(CHIME)?,
            select: config.load_chunk(SELECT)?,
            menu_music: StructuredMusic::repeat(MENU)?.into_rc(),
            title_music: StructuredMusic::repeat(TITLE)?.into_rc(),
            high_score_music: StructuredMusic::new(HIGH_SCORE_INTRO, HIGH_SCORE_REPEAT)?.into_rc(),
        })
    }

    pub fn play_chime(&self) -> Result<(), String> {
        self.chime.play()
    }

    pub fn play_select(&self) -> Result<(), String> {
        self.select.play()
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
