use std::cell::UnsafeCell;
use std::rc::Rc;
use sdl2::mixer::{Chunk, Music};
use sdl2::rwops::RWops;
use sdl2::sys::mixer;
use sdl2::get_error;

use crate::config::AudioConfig;
use crate::game::event::GameEvent;

static mut NEXT_MUSIC: Option<Rc<StructuredMusic>> = None;

pub struct StructuredMusic {
    intro: Option<Music<'static>>,
    repeating: Music<'static>,
    loops: i32
}

impl StructuredMusic {
    pub fn new(intro: &'static [u8], repeating: &'static [u8]) -> Result<Self, String> {
        Ok(Self {
            intro: Some(Music::from_static_bytes(intro)?),
            repeating: Music::from_static_bytes(repeating)?,
            loops: -1
        })
    }

    pub fn once(repeating: &'static [u8]) -> Result<Self, String> {
        Ok(Self {
            intro: None,
            repeating: Music::from_static_bytes(repeating)?,
            loops: 1
        })
    }

    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn play(music: &Rc<StructuredMusic>) -> Result<(), String> {
        if let Some(intro) = music.intro.as_ref() {
            Music::unhook_finished();
            intro.play(0)?;
            unsafe {
                NEXT_MUSIC = Some(music.clone());
            }
            Music::hook_finished(Self::play_next);
            Ok(())
        } else {
            music.repeating.play(music.loops)
        }
    }

    pub fn maybe_play(music: Option<&Rc<StructuredMusic>>) -> Result<(), String> {
        if let Some(music) = music {
            StructuredMusic::play(music)
        } else {
            Music::halt();
            Ok(())
        }
    }

    fn play_next() {
        Music::unhook_finished();
        unsafe {
            if let Some(music) = NEXT_MUSIC.as_ref() {
                music.repeating.play(-1).unwrap();
            }
        }
    }
}

pub trait LoadSound {
    fn load_chunk(&self, buffer: &[u8]) -> Result<Chunk, String>;
}

impl LoadSound for AudioConfig {
    fn load_chunk(&self, buffer: &[u8]) -> Result<Chunk, String> {
        let raw = unsafe { mixer::Mix_LoadWAV_RW(RWops::from_bytes(buffer)?.raw(), 0) };
        if raw.is_null() {
            Err(get_error())
        } else {
            let mut chunk = Chunk { raw, owned: true };
            chunk.set_volume(self.effects_volume());
            Ok(chunk)
        }
    }
}

pub trait Playable {
    fn play(&self) -> Result<(), String>;
}

impl Playable for Chunk {
    fn play(&self) -> Result<(), String> {
        // TODO ignore cannot play sound
        sdl2::mixer::Channel::all().play(self, 0)?;
        Ok(())
    }
}

pub struct AudioTheme {
    game_music: Option<Rc<StructuredMusic>>,
    game_over_music: Option<Rc<StructuredMusic>>,
    next_level_music: Option<Rc<StructuredMusic>>,
    victory_music: Option<Rc<StructuredMusic>>,
    move_pill: Chunk,
    rotate: Chunk,
    drop: Chunk,
    destroy_virus: Chunk,
    destroy_virus_combo: Chunk,
    destroy_vitamin: Chunk,
    destroy_vitamin_combo: Chunk,
    paused: Chunk,
    speed_level_up: Chunk,
    receive_garbage: Chunk,
}

impl AudioTheme {
    pub fn new(
        config: AudioConfig,
        pill_move: &[u8],
        rotate: &[u8],
        drop: &[u8],
        destroy_virus: &[u8],
        destroy_virus_combo: &[u8],
        destroy_vitamin: &[u8],
        destroy_vitamin_combo: &[u8],
        paused: &[u8],
        speed_level_up: &[u8],
        receive_garbage: &[u8]
    ) -> Result<Self, String> {
        Ok(Self {
            game_music: None,
            game_over_music: None,
            next_level_music: None,
            victory_music: None,
            move_pill: config.load_chunk(pill_move)?,
            rotate: config.load_chunk(rotate)?,
            drop: config.load_chunk(drop)?,
            destroy_virus: config.load_chunk(destroy_virus)?,
            destroy_virus_combo: config.load_chunk(destroy_virus_combo)?,
            destroy_vitamin: config.load_chunk(destroy_vitamin)?,
            destroy_vitamin_combo: config.load_chunk(destroy_vitamin_combo)?,
            paused: config.load_chunk(paused)?,
            speed_level_up: config.load_chunk(speed_level_up)?,
            receive_garbage: config.load_chunk(receive_garbage)?,
        })
    }

    pub fn with_game_music(mut self, intro: &'static [u8], repeating: &'static [u8]) -> Result<Self, String> {
        self.game_music = Some(StructuredMusic::new(intro, repeating)?.into_rc());
        Ok(self)
    }

    pub fn with_game_over_music(mut self, intro: &'static [u8], repeating: &'static [u8]) -> Result<Self, String> {
        self.game_over_music = Some(StructuredMusic::new(intro, repeating)?.into_rc());
        Ok(self)
    }

    pub fn with_next_level_music(mut self, intro: &'static [u8], repeating: &'static [u8]) -> Result<Self, String> {
        self.next_level_music = Some(StructuredMusic::new(intro, repeating)?.into_rc());
        Ok(self)
    }

    pub fn with_once_next_level_music(mut self, music: &'static [u8]) -> Result<Self, String> {
        self.next_level_music = Some(StructuredMusic::once(music)?.into_rc());
        Ok(self)
    }

    pub fn with_victory_music(mut self, intro: &'static [u8], repeating: &'static [u8]) -> Result<Self, String> {
        self.victory_music = Some(StructuredMusic::new(intro, repeating)?.into_rc());
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

    pub fn pause_music(&self) {
        Music::pause();
    }

    pub fn receive_event(&self, event: GameEvent) -> Result<(), String> {
        match event {
            GameEvent::Move => self.move_pill.play(),
            GameEvent::Rotate => self.rotate.play(),
            GameEvent::Lock { .. } | GameEvent::DropGarbage => self.drop.play(),
            GameEvent::Destroy { blocks, is_combo, .. } => {
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
            },
            GameEvent::ReceivedGarbage { .. } => self.receive_garbage.play(),
            GameEvent::SpeedLevelUp => self.speed_level_up.play(),
            GameEvent::Paused => {
                Music::pause();
                self.paused.play()
            },
            GameEvent::UnPaused => {
                Music::resume();
                Ok(())
            }
            _ => Ok(())
        }
    }
}

