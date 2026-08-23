use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::theme::font::{FontRenderOptions, FontSprite, MetricSnips};
use crate::theme::geometry::VISIBLE_BUFFER;
use crate::theme::retro::{retro_theme, RetroThemeOptions};
use crate::theme::sound::SoundThemeOptions;
use crate::theme::sprite_sheet::TetrominoSpriteSheetMeta;
use crate::theme::{Theme, ThemeName};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

const SPRITES: &[u8] = include_bytes!("sprites.png");
const BACKGROUND_FILE: &[u8] = include_bytes!("background.png");
const BOARD_FILE: &[u8] = include_bytes!("board.png");
const GAME_OVER_FILE: &[u8] = include_bytes!("game-over.png");

const GAME_OVER_SOUND: &[u8] = include_bytes!("game-over.ogg");
const LEVEL_UP_SOUND: &[u8] = include_bytes!("level-up.ogg");
const CLEAR_SOUND: &[u8] = include_bytes!("line-clear.ogg");
const LOCK_SOUND: &[u8] = include_bytes!("lock.ogg");
const MOVE_SOUND: &[u8] = include_bytes!("move.ogg");
const MUSIC: &[u8] = include_bytes!("music.ogg");
const PAUSE_SOUND: &[u8] = include_bytes!("pause.ogg");
const ROTATE_SOUND: &[u8] = include_bytes!("rotate.ogg");
const SEND_GARBAGE_SOUND: &[u8] = include_bytes!("send-garbage.ogg");
const STACK_DROP_SOUND: &[u8] = include_bytes!("stack-drop.ogg");
const TETRIS_SOUND: &[u8] = include_bytes!("tetris.ogg");
const VICTORY_SOUND: &[u8] = include_bytes!("victory.ogg");

const ALPHA_WIDTH: u32 = 7;
const ALPHA_HEIGHT: u32 = 8;
const BLOCK_PIXELS: u32 = 8;
const BUFFER_PIXELS: u32 = VISIBLE_BUFFER * BLOCK_PIXELS;

fn mino(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_PIXELS as i32, j * BLOCK_PIXELS as i32)
}

fn char_snip(row: i32, col: i32) -> Rect {
    let point = Point::new(col * 8, 35 + row * 9);
    Rect::new(point.x(), point.y(), ALPHA_WIDTH, ALPHA_HEIGHT)
}

pub fn snes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config,
) -> Result<Theme<'a>, String> {
    let mut font_sprites = Vec::with_capacity(24 * 2 + 10);
    for i in 0..10 {
        let snip = char_snip(0, i);
        font_sprites.push(FontSprite::new(char::from_u32('0' as u32 + i as u32).unwrap(), snip));
    }
    for i in 0..24 {
        let snip = char_snip(1, i);
        // uppercase
        font_sprites.push(
            FontSprite::new(char::from_u32('A' as u32 + i as u32).unwrap(), snip)
        );
        // lowercase
        font_sprites.push(
            FontSprite::new(char::from_u32('a' as u32 + i as u32).unwrap(), snip)
        );
    }

    let options = RetroThemeOptions::new(
        ThemeName::Snes,
        TetrominoSpriteSheetMeta::new(
            SPRITES,
            BLOCK_PIXELS,
            (mino(1, 1), mino(1, 0)),
            (mino(3, 1), mino(3, 0)),
            (mino(2, 1), mino(2, 0)),
            (mino(0, 1), mino(0, 0)),
            (mino(2, 1), mino(2, 0)),
            (mino(0, 1), mino(0, 0)),
            (mino(3, 1), mino(3, 0)),
            mino(0, 0),
            0x50,
        ),
        BACKGROUND_FILE,
        BOARD_FILE,
        GAME_OVER_FILE,
        [
            Rect::new(168, 17 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 58 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 82 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 106 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 130 + BUFFER_PIXELS as i32, 32, 32),
        ],
        Rect::new(19, 133 + BUFFER_PIXELS as i32, 32, 32),
        FontRenderOptions::Sprites {
            file_bytes: SPRITES,
            sprites: font_sprites,
            spacing: 1,
        },
        MetricSnips::zero_fill((7, 22), 6),
        MetricSnips::zero_fill((23, 62), 3),
        MetricSnips::zero_fill((23, 98), 4),
        Point::new(62, 0),
        Point::new(8, 0),
        Color::RGB(0x74, 0x74, 0x74),
        DestroyAnimationType::Sweep,
        GameOverAnimationType::CurtainDown,
        SoundThemeOptions::default(
            config.audio,
            MUSIC,
            MOVE_SOUND,
            ROTATE_SOUND,
            LOCK_SOUND,
            SEND_GARBAGE_SOUND,
            [CLEAR_SOUND, CLEAR_SOUND, CLEAR_SOUND, TETRIS_SOUND],
            LEVEL_UP_SOUND,
            GAME_OVER_SOUND,
            PAUSE_SOUND,
            VICTORY_SOUND,
        )
        .with_stack_drop(STACK_DROP_SOUND),
    );
    retro_theme(canvas, texture_creator, options)
}
