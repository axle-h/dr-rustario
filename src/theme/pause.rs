use crate::font::{FontTexture, FontType};
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum::RGBA8888;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;

pub struct PausedScreen<'a> {
    texture: Texture<'a>,
}

impl<'a> PausedScreen<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        ttf: &Sdl2TtfContext,
        texture_creator: &'a TextureCreator<WindowContext>,
        (window_width, window_height): (u32, u32),
    ) -> Result<Self, String> {
        let font = FontType::Retro.load(ttf, window_width / 16)?;
        let font_texture = FontTexture::from_string(&font, texture_creator, "pause", Color::WHITE)?;
        let font_rect = Rect::from_center(
            (window_width as i32 / 2, window_height as i32 / 2),
            font_texture.width,
            font_texture.height,
        );

        let mut texture = texture_creator
            .create_texture_target(RGBA8888, window_width, window_height)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        canvas
            .with_texture_canvas(&mut texture, |c| {
                c.set_draw_color(Color::RGBA(0, 0, 0, 0xdd));
                c.clear();
                c.copy(&font_texture.texture, None, font_rect).unwrap();
            })
            .map_err(|e| e.to_string())?;
        Ok(Self { texture })
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas.copy(&self.texture, None, None)
    }
}
