use crate::audio::{self, Sound};
use crate::config::AudioConfig;
use crate::game::event::GameEvent;
pub use engine::audio::theme::{LoadSound, StructuredMusic};
use std::rc::Rc;

pub struct AudioTheme {
    game_music: Option<Rc<StructuredMusic>>,
    game_over_music: Option<Rc<StructuredMusic>>,
    next_level_music: Option<Rc<StructuredMusic>>,
    victory_music: Option<Rc<StructuredMusic>>,
    move_pill: Sound,
    rotate: Sound,
    drop: Sound,
    destroy_virus: Sound,
    destroy_virus_combo: Sound,
    destroy_vitamin: Sound,
    destroy_vitamin_combo: Sound,
    paused: Sound,
    speed_level_up: Sound,
    receive_garbage: Sound,
    next_level_jingle: Sound,
    hard_drop: Option<Sound>,
}

impl AudioTheme {
    pub fn new<H: Into<Option<&'static [u8]>>>(
        config: AudioConfig,
        pill_move: &'static [u8],
        rotate: &'static [u8],
        drop: &'static [u8],
        destroy_virus: &'static [u8],
        destroy_virus_combo: &'static [u8],
        destroy_vitamin: &'static [u8],
        destroy_vitamin_combo: &'static [u8],
        paused: &'static [u8],
        speed_level_up: &'static [u8],
        receive_garbage: &'static [u8],
        next_level_jingle: &'static [u8],
        hard_drop: H,
    ) -> Result<Self, String> {
        let mut next_level_jingle = config.load_sound(next_level_jingle)?;
        next_level_jingle.set_volume(next_level_jingle.volume() / 2);

        Ok(Self {
            game_music: None,
            game_over_music: None,
            next_level_music: None,
            victory_music: None,
            move_pill: config.load_sound(pill_move)?,
            rotate: config.load_sound(rotate)?,
            drop: config.load_sound(drop)?,
            destroy_virus: config.load_sound(destroy_virus)?,
            destroy_virus_combo: config.load_sound(destroy_virus_combo)?,
            destroy_vitamin: config.load_sound(destroy_vitamin)?,
            destroy_vitamin_combo: config.load_sound(destroy_vitamin_combo)?,
            paused: config.load_sound(paused)?,
            speed_level_up: config.load_sound(speed_level_up)?,
            receive_garbage: config.load_sound(receive_garbage)?,
            next_level_jingle,
            hard_drop: hard_drop.into().map(|c| config.load_sound(c).unwrap()),
        })
    }

    pub fn with_game_music(
        mut self,
        intro: &'static [u8],
        repeating: &'static [u8],
    ) -> Result<Self, String> {
        self.game_music = Some(StructuredMusic::new(intro, repeating)?.into_rc());
        Ok(self)
    }

    pub fn with_game_over_music<R: Into<Option<&'static [u8]>>>(
        mut self,
        music: &'static [u8],
        repeating: R,
    ) -> Result<Self, String> {
        if let Some(repeating) = repeating.into() {
            self.game_over_music = Some(StructuredMusic::new(music, repeating)?.into_rc());
        } else {
            self.game_over_music = Some(StructuredMusic::once(music)?.into_rc());
        }
        Ok(self)
    }

    pub fn with_next_level_music<R: Into<Option<&'static [u8]>>>(
        mut self,
        music: &'static [u8],
        repeating: R,
    ) -> Result<Self, String> {
        if let Some(repeating) = repeating.into() {
            self.next_level_music = Some(StructuredMusic::new(music, repeating)?.into_rc());
        } else {
            self.next_level_music = Some(StructuredMusic::once(music)?.into_rc());
        }
        Ok(self)
    }

    pub fn with_victory_music<R: Into<Option<&'static [u8]>>>(
        mut self,
        music: &'static [u8],
        repeating: R,
    ) -> Result<Self, String> {
        if let Some(repeating) = repeating.into() {
            self.victory_music = Some(StructuredMusic::new(music, repeating)?.into_rc());
        } else {
            self.victory_music = Some(StructuredMusic::repeat(music)?.into_rc());
        }
        Ok(self)
    }

    pub fn play_game_music(&self) -> Result<(), String> {
        StructuredMusic::maybe_play(self.game_music.as_ref())
    }

    pub fn fade_in_game_music(&self) -> Result<(), String> {
        // TODO fade in
        StructuredMusic::maybe_play(self.game_music.as_ref())
    }

    pub fn play_game_over_music(&self) -> Result<(), String> {
        StructuredMusic::maybe_play(self.game_over_music.as_ref())
    }

    pub fn play_next_level_music(&self) -> Result<(), String> {
        StructuredMusic::maybe_play(self.next_level_music.as_ref())
    }

    pub fn play_victory_music(&self) -> Result<(), String> {
        StructuredMusic::maybe_play(self.victory_music.as_ref())
    }

    pub fn pause_music(&self) -> Result<(), String> {
        audio::pause_music()
    }

    pub fn play_next_level_jingle(&self) -> Result<(), String> {
        self.next_level_jingle.play()
    }

    pub fn receive_event(&self, event: GameEvent) -> Result<(), String> {
        match event {
            GameEvent::Move => self.move_pill.play(),
            GameEvent::Rotate => self.rotate.play(),
            GameEvent::Lock { .. } | GameEvent::DropGarbage => self.drop.play(),
            GameEvent::HardDrop { .. } => {
                self.hard_drop.as_ref().map(|c| c.play()).unwrap_or(Ok(()))
            }
            GameEvent::Destroy {
                blocks, is_combo, ..
            } => {
                if blocks.iter().any(|b| b.is_virus) {
                    if is_combo {
                        self.destroy_virus_combo.play()
                    } else {
                        self.destroy_virus.play()
                    }
                } else {
                    if is_combo {
                        self.destroy_vitamin_combo.play()
                    } else {
                        self.destroy_vitamin.play()
                    }
                }
            }
            GameEvent::ReceivedGarbage { .. } => self.receive_garbage.play(),
            GameEvent::SpeedLevelUp => self.speed_level_up.play(),
            GameEvent::Paused => {
                audio::pause_music()?;
                self.paused.play()
            }
            GameEvent::UnPaused => audio::resume_music(),
            _ => Ok(()),
        }
    }
}
