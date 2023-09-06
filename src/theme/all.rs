use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use crate::config::Config;
use crate::theme::nes::nes_theme;
use crate::theme::snes::snes_theme;
use crate::theme::Theme;

pub struct AllThemes<'a> {
    nes: Theme<'a>,
    snes: Theme<'a>
}

impl<'a> AllThemes<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf: &Sdl2TtfContext,
        config: Config,
        window_height: u32,
    ) -> Result<Self, String> {
        let nes = nes_theme(canvas, texture_creator, config)?;
        let snes = snes_theme(canvas, texture_creator, config)?;
        Ok(Self { nes, snes })
    }

    pub fn all(&self) -> Vec<&Theme<'a>> {
        vec![&self.nes, &self.snes]
    }
}