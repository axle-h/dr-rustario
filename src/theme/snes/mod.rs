use std::collections::HashMap;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::config::Config;
use crate::theme::retro::{retro_theme, RetroThemeOptions};
use crate::theme::sprite_sheet::{BlockPoints, pills, VitaminSpriteSheetData};
use crate::theme::{Theme, ThemeName};
use crate::theme::font::{alpha_sprites, FontRenderOptions, MetricSnips};
use crate::theme::geometry::BottleGeometry;
use crate::theme::scene::SceneType;
use crate::theme::sound::AudioTheme;

mod sprites {
    pub const SPRITES: &[u8] = include_bytes!("sprites.png");
    pub const BACKGROUND: &[u8] = include_bytes!("background.png");
    pub const BOTTLES: &[u8] = include_bytes!("bottles.png");
    pub const FONT: &[u8] = include_bytes!("font.png");
    pub const MATCH_END: &[u8] = include_bytes!("match-end.png");
    pub const BACKGROUND_TILE: &[u8] = include_bytes!("background-tile.png");
}
mod sound {
    pub const FEVER_INTRO: &[u8] = include_bytes!("fever-intro.ogg");
    pub const FEVER_REPEAT: &[u8] = include_bytes!("fever-repeat.ogg");
    pub const FEVER_NEXT_LEVEL: &[u8] = include_bytes!("fever-next-level.ogg");
    pub const DESTROY_VIRUS: &[u8] = include_bytes!("destroy-virus.ogg");
    pub const DESTROY_VIRUS_COMBO: &[u8] = include_bytes!("destroy-virus-combo.ogg");
    pub const DESTROY_VITAMIN: &[u8] = include_bytes!("destroy-vitamin.ogg");
    pub const DESTROY_VITAMIN_COMBO: &[u8] = include_bytes!("destroy-vitamin-combo.ogg");
    pub const GAME_OVER_INTRO: &[u8] = include_bytes!("game-over-intro.ogg");
    pub const GAME_OVER_REPEAT: &[u8] = include_bytes!("game-over-repeat.ogg");
    pub const RECEIVE_GARBAGE: &[u8] = include_bytes!("garbage.ogg");
    pub const SPEED_LEVEL_UP: &[u8] = include_bytes!("speed-level-up.ogg");
    pub const DROP: &[u8] = include_bytes!("drop.ogg");
    pub const MOVE_PILL: &[u8] = include_bytes!("move.ogg");
    pub const PAUSE: &[u8] = include_bytes!("pause.ogg");
    pub const ROTATE: &[u8] = include_bytes!("rotate.ogg");
    pub const VICTORY_INTRO: &[u8] = include_bytes!("victory-intro.ogg");
    pub const VICTORY_REPEAT: &[u8] = include_bytes!("victory-repeat.ogg");
}

const BLOCK_SIZE: u32 = 8;
const DR_NORMAL_WIDTH: u32 = 30;
const DR_NORMAL_HEIGHT: u32 = 38;
const ALPHA_WIDTH: u32 = 7;
const ALPHA_HEIGHT: u32 = 8;

fn block(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_SIZE as i32, j * BLOCK_SIZE as i32)
}
fn dr_normal(i: i32) -> Point {
    Point::new(i * DR_NORMAL_WIDTH as i32, 30)
}

fn color(j: i32) -> BlockPoints {
    BlockPoints::new(
        [block(4, j), block(5, j)],
        [block(11, j), block(10, j)],
        [block(5, j), block(4, j)],
        [block(10, j), block(11, j)],
        block(3, j),
        vec![block(0, j), block(1, j)],
        vec![block(2, j)]
    )
}

fn char_snip(col: i32) -> Point {
    Point::new(col * (ALPHA_WIDTH as i32 + 1), 0)
}

fn match_end(i: i32, j: i32) -> Point {
    Point::new(i * 65 + 1, j * 129 + 1)
}

pub fn snes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config
) -> Result<Theme<'a>, String> {
    let scene = SceneType::Tile { texture: sprites::BACKGROUND_TILE };

    let options = RetroThemeOptions {
        name: ThemeName::Snes,
        scene_low: scene.clone(),
        scene_medium: scene.clone(),
        scene_high: scene,
        sprites: VitaminSpriteSheetData::new(
            sprites::SPRITES,
            pills(
                BLOCK_SIZE * 2, BLOCK_SIZE,
                block(4, 0), block(6, 0), block(8, 0),
                block(4, 1), block(6, 1), block(8, 1),
                block(4, 2), block(6, 2), block(8, 2)
            ),
            color(0),
            color(2),
            color(1),
            BLOCK_SIZE,
            0x50,
            vec![dr_normal(1), dr_normal(0), dr_normal(2)],
            (DR_NORMAL_WIDTH, DR_NORMAL_HEIGHT),
            vec![Point::new(46, 69)],
            (34, 36),
            vec![Point::new(81, 69), Point::new(105, 69)],
            (24, 36),
            vec![Point::new(0, 69), Point::new(23, 69)],
            (23, 36)
        ),
        geometry: BottleGeometry::new(BLOCK_SIZE, 0, (7, 39)),
        audio: AudioTheme::new(
            config.audio, sound::MOVE_PILL, sound::ROTATE, sound::DROP,
            sound::DESTROY_VIRUS, sound::DESTROY_VIRUS_COMBO, sound::DESTROY_VITAMIN, sound::DESTROY_VITAMIN_COMBO,
            sound::PAUSE, sound::SPEED_LEVEL_UP, sound::RECEIVE_GARBAGE
        )?
            .with_game_music(sound::FEVER_INTRO, sound::FEVER_REPEAT)?
            .with_game_over_music(sound::GAME_OVER_INTRO, sound::GAME_OVER_REPEAT)?
            .with_once_next_level_music(sound::FEVER_NEXT_LEVEL)?
            .with_victory_music(sound::VICTORY_INTRO, sound::VICTORY_REPEAT)?,
        font: FontRenderOptions::Sprites {
            file_bytes: sprites::FONT,
            sprites: alpha_sprites(
                (0..10)
                    .map(|i| char_snip(i))
                    .collect::<Vec<Point>>()
                    .try_into()
                    .unwrap(),
                ALPHA_WIDTH,
                ALPHA_HEIGHT,
            ),
            spacing: 1,
        },
        bottles_file: sprites::BOTTLES,
        bottle_low: Point::new(0, 0),
        bottle_medium: Point::new(0, 0),
        bottle_high: Point::new(0, 0),
        bottle_width: 79,
        bottle_height: 175,
        background_file: sprites::BACKGROUND,
        bottle_point: Point::new(0, 0),
        match_end_file: sprites::MATCH_END,
        game_over_points: vec![match_end(0, 0), match_end(1, 0)],
        next_level_points: vec![
            match_end(2, 0), match_end(3, 0), match_end(4, 0),
            match_end(0, 1), match_end(1, 1), match_end(2, 1), match_end(3, 1), match_end(4, 1)
        ],
        dr_normal_point: Point::new(99, 29),
        dr_wait_point: Point::new(106, 31),
        dr_game_over_point: Point::new(100, 31),
        dr_victory_point: Point::new(105, 31),
        dr_hand_point: Point::new(103, 22),
        hold_point: Point::new(125, 18),
        peek_point: Point::new(96, 46),
        peek_offset: 10,
        peek_max: 2,
        peek_scale: Some(0.82),
        score: MetricSnips::zero_fill((91, 110), 9999999),
        virus_level: MetricSnips::zero_fill((123, 131), 99),
        virus_count: MetricSnips::zero_fill((123, 152), 99),
    };

    retro_theme(canvas, texture_creator, options)
}