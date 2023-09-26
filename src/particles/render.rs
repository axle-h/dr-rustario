use crate::particles::meta::ParticleSprite;
use crate::particles::scale::Scale;
use crate::particles::source::ParticleSource;
use crate::particles::Particles;

use crate::theme::sprite_sheet::FlatVitaminSpriteSheet;
use crate::theme::{Theme, ThemeName};

use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use std::collections::HashMap;
use std::time::Duration;
use strum::IntoEnumIterator;

const SPRITES: &[u8] = include_bytes!("sprites.png");
const BASE_SCALE: f64 = 0.05;

pub struct ParticleRender<'a> {
    scale: Scale,
    sprites: Texture<'a>,
    sprite_snips: HashMap<ParticleSprite, Rect>,
    particles: Particles,
    vitamin_sprites: HashMap<ThemeName, FlatVitaminSpriteSheet<'a>>,
}

impl<'a> ParticleRender<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        particles: Particles,
        texture_creator: &'a TextureCreator<WindowContext>,
        scale: Scale,
        all_themes: Vec<&Theme<'a>>,
    ) -> Result<Self, String> {
        let mut sprites = texture_creator.load_texture_bytes(SPRITES)?;
        sprites.set_blend_mode(BlendMode::Blend);

        let sprite_snips = ParticleSprite::iter()
            .filter(|s| s.snip().is_some())
            .map(|s| (s, s.snip().unwrap()))
            .collect();

        let vitamin_sprites = all_themes
            .into_iter()
            .map(|theme| (theme.name(), theme.sprites().flatten(canvas, texture_creator).unwrap()))
            .collect();

        Ok(Self {
            scale,
            particles,
            sprites,
            sprite_snips,
            vitamin_sprites,
        })
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn add_source(&mut self, source: Box<dyn ParticleSource>) {
        self.particles.sources.push(source);
    }

    pub fn update(&mut self, delta: Duration) {
        self.particles.update(delta)
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        for particle in self.particles.particles() {
            let (r, g, b): (u8, u8, u8) = particle.color().into();
            self.sprites.set_color_mod(r, g, b);
            if particle.alpha() < 1.0 {
                self.sprites
                    .set_alpha_mod((255.0 * particle.alpha()).round() as u8);
            } else {
                self.sprites.set_alpha_mod(255);
            }

            let point = self.scale.point_to_render_space(particle.position());

            if let Some(snip) = self.sprite_snips.get(&particle.sprite()) {
                let scale = BASE_SCALE * particle.size();
                let rect = Rect::from_center(
                    point,
                    (scale * snip.width() as f64).round() as u32,
                    (scale * snip.height() as f64).round() as u32,
                );
                canvas.copy(&self.sprites, *snip, rect)?;
            } else if let ParticleSprite::Pill(theme, shape) = particle.sprite() {
                // tetromino particle
                let scale = particle.size();

                let sprite_sheet = &self.vitamin_sprites[&theme];
                let snip = sprite_sheet.snip(shape);
                let rect = Rect::from_center(
                    point,
                    (scale * snip.width() as f64).round() as u32,
                    (scale * snip.height() as f64).round() as u32,
                );

                if particle.rotation() > 0.0 || particle.rotation() < 0.0 {
                    canvas.copy_ex(
                        sprite_sheet.texture(),
                        snip,
                        rect,
                        particle.rotation(),
                        None,
                        false,
                        false,
                    )?;
                } else {
                    canvas.copy(sprite_sheet.texture(), snip, rect)?;
                }
            } else {
                unreachable!()
            }
        }
        Ok(())
    }
}