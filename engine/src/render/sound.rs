use crate::audio::theme::{LoadSound, StructuredMusic};
use crate::audio::{self, Sound};
use crate::config::AudioConfig;
use crate::game::GameEvent;
use std::collections::HashMap;
use std::rc::Rc;

/// Which sound effect a theme plays for a game event. `Clear(class)` lets a game grade its
/// clears (a tetris vs a single, a virus vs a vitamin) without the theme knowing the rules:
/// see [`crate::render::GameRender::clear_class`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SfxKey {
    Move,
    Rotate,
    Lock,
    /// loose blocks settled after a clear
    Settle,
    HardDrop,
    Hold,
    Clear(u16),
    AttackSent,
    AttackReceived,
    SpeedUp,
    Paused,
    StageComplete,
}

pub struct AudioTheme {
    sfx: HashMap<SfxKey, Sound>,
    game_music: Option<Rc<StructuredMusic>>,
    game_over_music: Option<Rc<StructuredMusic>>,
    next_stage_music: Option<Rc<StructuredMusic>>,
    victory_music: Option<Rc<StructuredMusic>>,
}

impl AudioTheme {
    pub fn new(config: AudioConfig, sfx: &[(SfxKey, &'static [u8])]) -> Result<Self, String> {
        let mut sounds = HashMap::new();
        for (key, bytes) in sfx.iter().copied() {
            let mut sound = config.load_sound(bytes)?;
            if key == SfxKey::StageComplete {
                // jingles play over music
                sound.set_volume(sound.volume() / 2);
            }
            sounds.insert(key, sound);
        }
        Ok(Self {
            sfx: sounds,
            game_music: None,
            game_over_music: None,
            next_stage_music: None,
            victory_music: None,
        })
    }

    fn music<R: Into<Option<&'static [u8]>>>(
        music: &'static [u8],
        repeating: R,
        loop_single: bool,
    ) -> Result<Rc<StructuredMusic>, String> {
        Ok(match repeating.into() {
            Some(repeating) => StructuredMusic::new(music, repeating)?,
            None if loop_single => StructuredMusic::repeat(music)?,
            None => StructuredMusic::once(music)?,
        }
        .into_rc())
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
        self.game_over_music = Some(Self::music(music, repeating, false)?);
        Ok(self)
    }

    pub fn with_next_stage_music<R: Into<Option<&'static [u8]>>>(
        mut self,
        music: &'static [u8],
        repeating: R,
    ) -> Result<Self, String> {
        self.next_stage_music = Some(Self::music(music, repeating, false)?);
        Ok(self)
    }

    pub fn with_victory_music<R: Into<Option<&'static [u8]>>>(
        mut self,
        music: &'static [u8],
        repeating: R,
    ) -> Result<Self, String> {
        self.victory_music = Some(Self::music(music, repeating, true)?);
        Ok(self)
    }

    pub fn play_game_music(&self) -> Result<(), String> {
        StructuredMusic::maybe_play(self.game_music.as_ref())
    }

    pub fn play_game_over_music(&self) -> Result<(), String> {
        StructuredMusic::maybe_play(self.game_over_music.as_ref())
    }

    pub fn play_next_stage_music(&self) -> Result<(), String> {
        StructuredMusic::maybe_play(self.next_stage_music.as_ref())
    }

    pub fn play_victory_music(&self) -> Result<(), String> {
        StructuredMusic::maybe_play(self.victory_music.as_ref())
    }

    pub fn play_stage_complete_jingle(&self) -> Result<(), String> {
        self.play(SfxKey::StageComplete)
    }

    pub fn pause_music(&self) -> Result<(), String> {
        audio::pause_music()
    }

    fn play(&self, key: SfxKey) -> Result<(), String> {
        match self.sfx.get(&key) {
            Some(sound) => sound.play(),
            None => Ok(()),
        }
    }

    /// `clear_class` grades a `Clear` event for this game, see [`SfxKey::Clear`]
    pub fn receive_event(&self, event: &GameEvent, clear_class: u16) -> Result<(), String> {
        match event {
            GameEvent::Move => self.play(SfxKey::Move),
            GameEvent::Rotate => self.play(SfxKey::Rotate),
            GameEvent::Hold => self.play(SfxKey::Hold),
            GameEvent::Lock { .. } => self.play(SfxKey::Lock),
            GameEvent::Settle => self.play(SfxKey::Settle),
            GameEvent::HardDrop { .. } => self.play(SfxKey::HardDrop),
            GameEvent::Clear { .. } => self.play(SfxKey::Clear(clear_class)),
            GameEvent::AttackSent(_) => self.play(SfxKey::AttackSent),
            GameEvent::AttackReceived { .. } => self.play(SfxKey::AttackReceived),
            GameEvent::SpeedUp => self.play(SfxKey::SpeedUp),
            GameEvent::Paused => {
                audio::pause_music()?;
                self.play(SfxKey::Paused)
            }
            GameEvent::UnPaused => audio::resume_music(),
            _ => Ok(()),
        }
    }
}
