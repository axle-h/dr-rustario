use sdl2::image::LoadTexture;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::theme::{Theme, ThemeName};
use crate::theme::geometry::BottleGeometry;
use crate::theme::sprite_sheet::{VitaminSpriteSheet, VitaminSpriteSheetData};

pub struct RetroThemeOptions {
    name: ThemeName,
    sprites: VitaminSpriteSheetData,
    geometry: BottleGeometry,
    bottle_file: &'static [u8],
    bottle_point: Point
}

impl RetroThemeOptions {
    pub fn new(name: ThemeName, sprites: VitaminSpriteSheetData, geometry: BottleGeometry, bottle_file: &'static [u8], bottle_point: Point) -> Self {
        Self { name, sprites, geometry, bottle_file, bottle_point }
    }
}

pub fn retro_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    options: RetroThemeOptions,
) -> Result<Theme<'a>, String> {
    let sprites = VitaminSpriteSheet::new(
        canvas,
        texture_creator,
        options.sprite_sheet,
        options.geometry.block_size(),
    )?;
    let bottle_texture = texture_creator.load_texture_bytes(options.bottle_file)?;
    let bottle_query = bottle_texture.query();
    let bottle_snip = Rect::new(
        options.bottle_point.x(),
        options.bottle_point.y(),
        bottle_query.width,
        bottle_query.height
    );

    Ok(
        Theme {
            name: options.name,
            sprites,
            geometry: options.geometry,
            bottle_texture,
            bottle_snip,
            background_size: (bottle_query.width, bottle_query.height) // todo
        }
    )
}