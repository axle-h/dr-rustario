use sdl2::rect::Point;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::theme::retro::{retro_theme, RetroThemeOptions};
use crate::theme::sprite_sheet::{BlockPoints, VitaminSpriteSheetData};
use crate::theme::{Theme, ThemeName};
use crate::theme::geometry::BottleGeometry;

const SPRITES: &[u8] = include_bytes!("sprites.png");
const BOTTLE_MEDIUM: &[u8] = include_bytes!("bottle-medium.png");
const BLOCK_SIZE: u32 = 9;

fn block(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_SIZE as i32, j * BLOCK_SIZE as i32)
}

fn color(j: i32) -> BlockPoints {
    BlockPoints::new(
        [block(0, j), block(1, j)],
        [block(2, j), block(3, j)],
        [block(1, j), block(0, j)],
        [block(3, j), block(2, j)],
        block(4, j),
        vec![block(6, j), block(7, j)],
        vec![block(8, j), block(9, j)],
        vec![block(5, j)]
    )
}

pub fn nes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<Theme<'a>, String> {
    let options = RetroThemeOptions::new(
        ThemeName::Nes,
        VitaminSpriteSheetData::new(
            SPRITES,
            color(0),
            color(2),
            color(1),
            BLOCK_SIZE,
            0x60
        ),
        BottleGeometry::new(9, -1, (0, 0)),
        BOTTLE_MEDIUM,
Point::new(0, 0), // TODO
    );

    retro_theme(canvas, texture_creator, options)
}