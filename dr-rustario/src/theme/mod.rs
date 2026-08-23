use crate::game::cell::DrCell;
use engine::animate::frames::FrameAnimationType;
pub use engine::animate::AnimationMeta;
use std::time::Duration;

pub const RETRO_THROW: FrameAnimationType = FrameAnimationType::LinearWithPause {
    fps: 10,
    pause_for: Duration::from_millis(200),
    resume_from_frame: 0,
};
pub const NES_SNES_VICTORY: FrameAnimationType = FrameAnimationType::Linear { fps: 4 };
pub const N64_VICTORY: FrameAnimationType = FrameAnimationType::LinearWithPause {
    fps: 7,
    pause_for: Duration::from_millis(2000),
    resume_from_frame: 0,
};
pub const N64_GAME_OVER: FrameAnimationType = FrameAnimationType::LinearWithPause {
    fps: 7,
    pause_for: Duration::from_millis(2000),
    resume_from_frame: 18,
};
use engine::animate::PlayerAnimations;
use engine::game::CellId;

use crate::game::pill::VirusColor;
use crate::game::{Game, GameSpeed};
use crate::particles::particle::ParticleAnimationType;
use crate::theme::font::FontTheme;
use crate::theme::geometry::BottleGeometry;
use crate::theme::scene::SceneRender;
use crate::theme::sound::AudioTheme;
use crate::theme::sprite_sheet::{DrType, VitaminSpriteSheet};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};

pub mod all;
pub use engine::render::animation;
pub use engine::render::block_mask;
pub mod font;
pub mod geometry;
pub use engine::render::helper;
pub mod n64;
pub mod nes;
pub mod particle;
pub use engine::render::pause;
mod retro;
pub mod scene;
pub mod snes;
pub mod sound;
pub mod sprite_sheet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum ThemeName {
    #[default]
    Nes,
    Snes,
    N64,
    Particle,
}

fn particle_animation(animation_type: FrameAnimationType, frames: usize) -> ParticleAnimationType {
    match animation_type {
        FrameAnimationType::Static => ParticleAnimationType::Static,
        FrameAnimationType::Linear { fps } => ParticleAnimationType::Linear { frames, fps },
        FrameAnimationType::YoYo { fps } => ParticleAnimationType::YoYo { frames, fps },
        FrameAnimationType::LinearWithPause { fps, .. } => {
            ParticleAnimationType::Linear { frames, fps }
        }
    }
}

pub fn virus_particle_animation(meta: &AnimationMeta, color: VirusColor) -> ParticleAnimationType {
    let frames = meta
        .cell_idle_frames(CellId::from(DrCell::Virus(color)))
        .unwrap_or(1);
    particle_animation(meta.cell_idle_type, frames)
}

pub fn dr_particle_animation(meta: &AnimationMeta, dr_type: DrType) -> ParticleAnimationType {
    let mascot = meta.mascot.expect("dr rustario themes have a mascot");
    let (animation_type, frames) = match dr_type {
        DrType::Throw => (mascot.spawn_type, mascot.spawn_frames),
        DrType::GameOver => (mascot.game_over_type, mascot.game_over_frames),
        DrType::Victory => (mascot.victory_type, mascot.victory_frames),
        DrType::Idle => (mascot.idle_type, mascot.idle_frames),
    };
    particle_animation(animation_type, frames)
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
    peek_scale: Option<f64>,
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

    pub fn animation_meta(&self) -> &AnimationMeta {
        &self.animation_meta
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

    pub fn draw_background(
        &self,
        canvas: &mut WindowCanvas,
        game: &Game,
        animations: &PlayerAnimations,
    ) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();
        let (width, height) = self.background_size;
        canvas.copy(
            &self.background_texture,
            None,
            Rect::new(0, 0, width, height),
        )?;

        let metrics = game.metrics();
        if let Some(game_over) = animations.game_over().state() {
            self.sprites.draw_dr(
                canvas,
                DrType::GameOver,
                self.dr_game_over_point,
                game_over.mascot_frame().unwrap_or(0),
            )?;
        } else if let Some(victory) = animations.victory().state() {
            self.sprites.draw_dr(
                canvas,
                DrType::Victory,
                self.dr_victory_point,
                victory.mascot_frame().unwrap_or(0),
            )?;
        } else if let Some(interstitial) = animations.interstitial().state() {
            self.sprites.draw_dr(
                canvas,
                DrType::Victory,
                self.dr_victory_point,
                interstitial.mascot_frame().unwrap_or(0),
            )?;
        } else {
            let peek = metrics.queue();
            let mut peek_offset = 0;
            if let Some(spawn) = animations.spawn().state() {
                if self.dr_order_first {
                    self.sprites.draw_dr(
                        canvas,
                        DrType::Throw,
                        self.dr_throw_point,
                        spawn.mascot_frame().unwrap_or(0),
                    )?;
                    self.sprites.draw_pill(
                        canvas,
                        spawn.piece().into(),
                        spawn.throw_position(),
                        spawn.piece_rotate_angle_degrees(),
                        None,
                    )?;
                } else {
                    self.sprites.draw_pill(
                        canvas,
                        spawn.piece().into(),
                        spawn.throw_position(),
                        spawn.piece_rotate_angle_degrees(),
                        None,
                    )?;
                    self.sprites.draw_dr(
                        canvas,
                        DrType::Throw,
                        self.dr_throw_point,
                        spawn.mascot_frame().unwrap_or(0),
                    )?;
                }

                if let Some(spawn_peek_offset) = spawn.peek_offset() {
                    peek_offset = self.peek_offset
                        - (spawn_peek_offset * self.peek_offset as f64).round() as i32;
                }
            } else if self.dr_order_first {
                self.sprites.draw_dr(
                    canvas,
                    DrType::Idle,
                    self.dr_throw_point,
                    animations.mascot_idle_frame().unwrap_or(0),
                )?;
                self.sprites
                    .draw_pill(canvas, peek[0], self.dr_hand_point, None, None)?;
            } else {
                self.sprites
                    .draw_pill(canvas, peek[0], self.dr_hand_point, None, None)?;
                self.sprites.draw_dr(
                    canvas,
                    DrType::Idle,
                    self.dr_throw_point,
                    animations.mascot_idle_frame().unwrap_or(0),
                )?;
            }
            if let Some(hold) = metrics.hold() {
                self.sprites
                    .draw_pill(canvas, hold, self.hold_point, None, self.peek_scale)?;
            }
            for i in 0..self.peek_max.min(peek.len() as u32 - 1) {
                let point = self
                    .peek_point
                    .offset(0, peek_offset + i as i32 * self.peek_offset);
                self.sprites.draw_pill(
                    canvas,
                    peek[i as usize + 1],
                    point,
                    None,
                    self.peek_scale,
                )?;
            }
        }

        self.font.render_all(canvas, metrics)
    }

    pub fn draw_bottle(
        &self,
        canvas: &mut WindowCanvas,
        game: &Game,
        animations: &PlayerAnimations,
    ) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        let bottle_snip = match game.speed() {
            GameSpeed::Low => self.bottle_low_snip,
            GameSpeed::Medium => self.bottle_medium_snip,
            GameSpeed::High => self.bottle_high_snip,
        };
        let bottle_dest = Rect::new(0, 0, bottle_snip.width(), bottle_snip.height());
        canvas.copy(&self.bottles_texture, bottle_snip, bottle_dest)?;

        self.sprites
            .draw_bottle(canvas, game, &self.geometry, animations)?;
        if let Some(game_over_frame) = animations
            .game_over()
            .state()
            .and_then(|s| s.screen_frame())
        {
            canvas.copy(
                &self.match_end_texture,
                self.game_over_snips[game_over_frame],
                self.geometry.game_snip(),
            )?;
        } else if let Some(interstitial_frame) = animations
            .interstitial()
            .state()
            .map(|s| s.interstitial_frame())
        {
            canvas.copy(
                &self.match_end_texture,
                self.next_level_snips[interstitial_frame],
                self.geometry.game_snip(),
            )?;
        }

        Ok(())
    }
}
