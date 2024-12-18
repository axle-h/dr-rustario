use crate::game::event::GameEvent;
use crate::game::pill::VITAMIN_SPAWN_POINTS;
use crate::particles::prescribed::{
    PlayerParticleTarget, PlayerTargetedParticles, PrescribedParticles,
};
use crate::scale::Scale;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use std::time::Duration;
use sdl2::pixels::PixelFormatEnum::RGBA8888;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneType {
    Particles {
        base_color: Color,
    },
    Checkerboard {
        width: u32,
        height: u32,
        colors: [Color; 2],
    },
    Tile {
        texture: &'static [u8],
    },
}

impl SceneType {
    pub fn build<'a>(
        &self,
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<SceneRender<'a>, String> {
        SceneRender::new(canvas, texture_creator, self.clone())
    }
}

pub struct SceneRender<'a> {
    scene_type: SceneType,
    texture: Texture<'a>,
    rect_0: Rect,
}

impl<'a> SceneRender<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        scene_type: SceneType,
    ) -> Result<Self, String> {
        let texture = match scene_type {
            SceneType::Checkerboard {
                width,
                height,
                colors: [color1, color2],
            } => {
                let mut texture = texture_creator
                    .create_texture_target(RGBA8888, width * 2, height * 2)
                    .map_err(|e| e.to_string())?;
                canvas
                    .with_texture_canvas(&mut texture, |c| {
                        c.set_draw_color(Color::RGBA(0, 0, 0, 0));
                        c.clear();

                        c.set_draw_color(color1);
                        c.fill_rects(&[
                            Rect::new(0, 0, width, height),
                            Rect::new(width as i32, height as i32, width, height),
                        ])
                        .unwrap();
                        c.set_draw_color(color2);
                        c.fill_rects(&[
                            Rect::new(width as i32, 0, width, height),
                            Rect::new(0, height as i32, width, height),
                        ])
                        .unwrap();
                    })
                    .map_err(|e| e.to_string())?;
                texture
            }
            SceneType::Tile { texture } => texture_creator.load_texture_bytes(texture)?,
            // TODO this is dirty
            SceneType::Particles { .. } => texture_creator
                .create_texture_target(None, 1, 1)
                .map_err(|e| e.to_string())?,
        };

        let query = texture.query();
        Ok(Self {
            scene_type,
            texture,
            rect_0: Rect::new(0, 0, query.width, query.height),
        })
    }

    pub fn is_particles(&self) -> bool {
        matches!(self.scene_type, SceneType::Particles { .. })
    }

    pub fn emit_particles(&self, event: GameEvent) -> Option<PlayerTargetedParticles> {
        if let SceneType::Particles { base_color } = self.scene_type {
            match event {
                GameEvent::Spawned { player } => {
                    let target = PlayerParticleTarget::Blocks(VITAMIN_SPAWN_POINTS.to_vec());
                    let particles = PrescribedParticles::LightBurstUpAndOut { color: base_color };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::HardDrop {
                    player, vitamins, ..
                } => {
                    let target = PlayerParticleTarget::Vitamins(vitamins);
                    let particles = PrescribedParticles::BurstUp { color: base_color };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::SendGarbage { player, .. } => Some(
                    PrescribedParticles::PerimeterBurst { color: base_color }
                        .into_targeted(player, PlayerParticleTarget::Bottle),
                ),
                GameEvent::Lock {
                    player,
                    vitamins,
                    hard_or_soft_dropped,
                } if hard_or_soft_dropped => {
                    let target = PlayerParticleTarget::Vitamins(vitamins);
                    let particles = PrescribedParticles::BurstDown { color: base_color };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::ReceivedGarbage { player, garbage } => {
                    let target = PlayerParticleTarget::Garbage(garbage);
                    let particles = PrescribedParticles::BurstDown { color: base_color };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::Destroy { player, blocks, .. } => {
                    let target = PlayerParticleTarget::MaskedBlocks(blocks);
                    let particles = PrescribedParticles::FadeInLatticeBurstAndFall {
                        fade_in: Duration::from_millis(250),
                        color: base_color,
                    };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::Victory { player } => Some(
                    PrescribedParticles::PerimeterSpray { color: base_color }
                        .into_targeted(player, PlayerParticleTarget::Bottle),
                ),
                GameEvent::LevelComplete { player } => Some(
                    PrescribedParticles::PerimeterBurst { color: base_color }
                        .into_targeted(player, PlayerParticleTarget::Bottle),
                ),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, scale: &Scale) -> Result<(), String> {
        if self.is_particles() {
            return Ok(());
        }

        let (window_width, window_height) = scale.window_size();
        let mut rect = scale.scale_rect(self.rect_0);

        let (offset_x, repeat_x) = Self::offset_and_repeat(window_width, rect.width());
        let (offset_y, repeat_y) = Self::offset_and_repeat(window_height, rect.height());

        rect.offset(offset_x, offset_y);
        for x in 0..repeat_x {
            for y in 0..repeat_y {
                rect.reposition((
                    x * rect.width() as i32 + offset_x,
                    y * rect.height() as i32 + offset_y,
                ));
                canvas.copy(&self.texture, None, rect)?;
            }
        }
        Ok(())
    }

    fn offset_and_repeat(window_size: u32, tile_size: u32) -> (i32, i32) {
        let remainder = window_size as i32 % tile_size as i32;
        let offset = -remainder / 2;
        let repeat = window_size as i32 / tile_size as i32 + if remainder == 0 { 0 } else { 2 };
        (offset, repeat)
    }
}
