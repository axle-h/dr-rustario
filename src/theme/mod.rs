use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use crate::game::Game;
use crate::theme::geometry::BottleGeometry;
use crate::theme::sprite_sheet::VitaminSpriteSheet;

pub mod geometry;
pub mod sprite_sheet;
pub mod nes;
mod retro;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum ThemeName {
    #[default]
    Nes
}

pub struct Theme<'a> {
    name: ThemeName,
    sprites: VitaminSpriteSheet<'a>,
    geometry: BottleGeometry,
    bottle_texture: Texture<'a>,
    bottle_snip: Rect,
    background_size: (u32, u32)
}

impl<'a> Theme<'a> {
    pub fn name(&self) -> ThemeName {
        self.name
    }

    pub fn geometry(&self) -> &BoardGeometry {
        &self.geometry
    }

    pub fn background_size(&self) -> (u32, u32) {
        self.background_size
    }

    pub fn bottle_snip(&self) -> Rect {
        self.bottle_snip
    }

    pub fn draw_background(&self, canvas: &mut WindowCanvas, game: &Game) -> Result<(), String> {
        // todo
        Ok(())
    }

    pub fn draw_bottle(&self, canvas: &mut WindowCanvas, game: &Game) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        // todo to make sure we dont clobber the first column and row, we need to actually render the bottle background last
        canvas.copy(
            &self.bottle_texture,
            None,
            Rect::new(0, 0, self.bottle_snip.width(), self.bottle_snip.height()),
        )?;

        self.sprites.draw_bottle(canvas, game, &self.geometry)?;

        Ok(())
    }
}