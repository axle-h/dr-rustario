use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use crate::config::Config;
use crate::theme::particle::particle_theme;
use crate::theme::n64::n64_theme;
use crate::theme::nes::nes_theme;
use crate::theme::snes::snes_theme;
use crate::theme::{AnimationMeta, Theme};

#[derive(Copy, Clone, Debug)]
pub struct AllThemeMeta {
    pub nes: AnimationMeta,
    pub snes: AnimationMeta,
    pub n64: AnimationMeta,
    pub particle: AnimationMeta
}

pub struct AllThemes<'a> {
    nes: Theme<'a>,
    snes: Theme<'a>,
    n64: Theme<'a>,
    particle: Theme<'a>,
    meta: AllThemeMeta
}

impl<'a> AllThemes<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf: &Sdl2TtfContext,
        config: Config
    ) -> Result<Self, String> {
        let nes = nes_theme(canvas, texture_creator, config)?;
        let snes = snes_theme(canvas, texture_creator, config)?;
        let n64 = n64_theme(canvas, texture_creator, config)?;
        let particle = particle_theme(canvas, texture_creator, ttf, config)?;
        let meta = AllThemeMeta {
            nes: nes.animation_meta.clone(),
            snes: snes.animation_meta.clone(),
            n64: n64.animation_meta.clone(),
            particle: particle.animation_meta.clone()
        };
        Ok(Self { nes, snes, n64, particle, meta })
    }

    pub fn all(&self) -> Vec<&Theme<'a>> {
        vec![&self.nes, &self.snes, &self.n64, &self.particle]
    }

    pub fn meta(&self) -> AllThemeMeta {
        self.meta
    }
}