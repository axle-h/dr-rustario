use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use crate::animate::dr::DrAnimationType;
use crate::animate::PlayerAnimations;
use crate::animate::virus::VirusAnimationType;
use crate::game::{Game, GameSpeed};
use crate::game::geometry::Rotation;
use crate::game::pill::{VirusColor, VitaminOrdinal};
use crate::particles::particle::ParticleAnimationType;
use crate::theme::font::{FontRender, FontTheme, MetricSnips};
use crate::theme::geometry::BottleGeometry;
use crate::theme::scene::{SceneRender, SceneType};
use crate::theme::sound::AudioTheme;
use crate::theme::sprite_sheet::{DrType, VitaminSpriteSheet};

pub mod geometry;
pub mod sprite_sheet;
pub mod nes;
mod retro;
pub mod all;
pub mod sound;
pub mod font;
pub mod pause;
pub mod scene;
pub mod snes;
pub mod n64;
pub mod animation;
pub mod particle;
pub mod helper;
pub mod block_mask;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum ThemeName {
    #[default]
    Nes,
    Snes,
    N64,
    Particle
}

#[derive(Copy, Clone, Debug)]
pub struct AnimationMeta {
    pub virus_type: VirusAnimationType,
    pub red_virus_frames: usize,
    pub blue_virus_frames: usize,
    pub yellow_virus_frames: usize,
    pub vitamin_pop_frames: usize,
    pub virus_pop_frames: usize,
    pub throw_start: Point,
    pub throw_end: Point,
    pub dr_throw_type: DrAnimationType,
    pub dr_throw_frames: usize,
    pub dr_victory_type: DrAnimationType,
    pub dr_victory_frames: usize,
    pub dr_idle_type: DrAnimationType,
    pub dr_idle_frames: usize,
    pub dr_game_over_type: DrAnimationType,
    pub dr_game_over_frames: usize,
    pub game_over_screen_frames: usize,
    pub next_level_interstitial_frames: usize
}

impl AnimationMeta {
    pub fn virus_particle_animation(&self, color: VirusColor) -> ParticleAnimationType {
        let frames = match color {
            VirusColor::Yellow => self.yellow_virus_frames,
            VirusColor::Blue => self.blue_virus_frames,
            VirusColor::Red => self.red_virus_frames
        };
        match self.virus_type {
            VirusAnimationType::Linear { fps } => ParticleAnimationType::Linear { frames, fps },
            VirusAnimationType::YoYo { fps } => ParticleAnimationType::YoYo { frames, fps }
        }
    }
}

pub struct Theme<'a> {
    name: ThemeName,
    scene_low: SceneRender<'a>,
    scene_medium: SceneRender<'a>,
    scene_high: SceneRender<'a>,
    sprites: VitaminSpriteSheet<'a>,
    geometry: BottleGeometry,
    audio: AudioTheme,
    font: FontTheme<'a>,
    bottles_texture: Texture<'a>,
    bottle_low_snip: Rect,
    bottle_medium_snip: Rect,
    bottle_high_snip: Rect,
    background_texture: Texture<'a>,
    bottle_bg_snip: Rect,
    background_size: (u32, u32),
    dr_order_first: bool,
    dr_hand_point: Point,
    dr_throw_point: Point,
    dr_game_over_point: Point,
    dr_victory_point: Point,
    animation_meta: AnimationMeta,
    game_over_snips: Vec<Rect>,
    next_level_snips: Vec<Rect>,
    match_end_texture: Texture<'a>,
    hold_point: Point,
    peek_point: Point,
    peek_max: u32,
    peek_offset: i32,
    peek_scale: Option<f64>
}

impl<'a> Theme<'a> {
    pub fn name(&self) -> ThemeName {
        self.name
    }

    pub fn sprites(&self) -> &VitaminSpriteSheet<'a> {
        &self.sprites
    }

    pub fn scene(&self, speed: GameSpeed) -> &SceneRender<'a> {
        match speed {
            GameSpeed::Low => &self.scene_low,
            GameSpeed::Medium => &self.scene_medium,
            GameSpeed::High => &self.scene_high,
        }
    }

    pub fn animation_meta(&self) -> AnimationMeta {
        self.animation_meta
    }

    pub fn geometry(&self) -> &BottleGeometry {
        &self.geometry
    }

    pub fn background_size(&self) -> (u32, u32) {
        self.background_size
    }

    pub fn bottle_snip(&self) -> Rect {
        self.bottle_bg_snip
    }

    pub fn audio(&self) -> &AudioTheme {
        &self.audio
    }

    pub fn draw_background(&self, canvas: &mut WindowCanvas, game: &Game, animations: &PlayerAnimations) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();
        let (width, height) = self.background_size;
        canvas.copy(&self.background_texture, None, Rect::new(0, 0, width, height))?;

        let metrics = game.metrics();
        if let Some(game_over) = animations.game_over().state() {
            self.sprites.draw_dr(canvas, DrType::GameOver, self.dr_game_over_point, game_over.dr_frame())?;
        } else if let Some(victory) = animations.victory().state() {
            self.sprites.draw_dr(canvas, DrType::Victory, self.dr_victory_point, victory.dr_frame())?;
        } else if let Some(next_level_interstitial) = animations.next_level_interstitial().state() {
            self.sprites.draw_dr(canvas, DrType::Victory, self.dr_victory_point, next_level_interstitial.dr_frame())?;
        } else {
            let peek = metrics.queue();
            let mut peek_offset = 0;
            if let Some(spawn) = animations.throw().state() {
                if self.dr_order_first {
                    self.sprites.draw_dr(canvas, DrType::Throw, self.dr_throw_point, spawn.dr_throw_frame())?;
                    self.sprites.draw_pill(canvas, spawn.shape(), spawn.throw_position(), spawn.pill_rotate_angle_degrees(), None)?;
                } else {
                    self.sprites.draw_pill(canvas, spawn.shape(), spawn.throw_position(), spawn.pill_rotate_angle_degrees(), None)?;
                    self.sprites.draw_dr(canvas, DrType::Throw, self.dr_throw_point, spawn.dr_throw_frame())?;
                }

                if let Some(spawn_peek_offset) = spawn.peek_offset() {
                    peek_offset = self.peek_offset - (spawn_peek_offset * self.peek_offset as f64).round() as i32;
                }
            } else if self.dr_order_first {
                self.sprites.draw_dr(canvas, DrType::Idle, self.dr_throw_point, animations.idle().frame())?;
                self.sprites.draw_pill(canvas, peek[0], self.dr_hand_point, None, None)?;
            } else {
                self.sprites.draw_pill(canvas, peek[0], self.dr_hand_point, None, None)?;
                self.sprites.draw_dr(canvas, DrType::Idle, self.dr_throw_point, animations.idle().frame())?;
            }
            if let Some(hold) = metrics.hold() {
                self.sprites.draw_pill(canvas, hold, self.hold_point, None, self.peek_scale)?;
            }
            for i in 0..self.peek_max.min(peek.len() as u32 - 1) {
                let point = self.peek_point.offset(0, peek_offset + i as i32 * self.peek_offset);
                self.sprites.draw_pill(canvas, peek[i as usize + 1], point, None, self.peek_scale)?;
            }
        }

        self.font.render_all(canvas, metrics)
    }

    pub fn draw_bottle(&self, canvas: &mut WindowCanvas, game: &Game, animations: &PlayerAnimations) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        let bottle_snip = match game.speed() {
            GameSpeed::Low => self.bottle_low_snip,
            GameSpeed::Medium => self.bottle_medium_snip,
            GameSpeed::High => self.bottle_high_snip
        };
        canvas.copy(&self.bottles_texture, bottle_snip, self.bottle_bg_snip)?;

        self.sprites.draw_bottle(canvas, game, &self.geometry, animations)?;
        if let Some(game_over_frame) = animations.game_over().state().and_then(|s| s.game_over_screen_frame()) {
            canvas.copy(&self.match_end_texture, self.game_over_snips[game_over_frame], self.geometry.game_snip())?;
        } else if let Some(interstitial_frame) = animations.next_level_interstitial().state().map(|s| s.interstitial_frame()) {
            canvas.copy(&self.match_end_texture, self.next_level_snips[interstitial_frame], self.geometry.game_snip())?;
        }

        Ok(())
    }
}