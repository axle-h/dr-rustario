use std::collections::HashMap;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::animate::dr::DrAnimationType;
use crate::animate::virus::VirusAnimationType;
use crate::config::Config;
use crate::theme::retro::{retro_theme, RetroThemeOptions};
use crate::theme::sprite_sheet::{BlockPoints, DrSpriteSheetData, pills, VitaminSpriteSheetData};
use crate::theme::{Theme, ThemeName};
use crate::theme::font::{alpha_sprites, FontRenderOptions, FontThemeOptions, MetricSnips, ThemedNumeric};
use crate::theme::geometry::BottleGeometry;
use crate::theme::scene::SceneType;
use crate::theme::sound::AudioTheme;

mod sprites {
    pub const VITAMINS: &[u8] = include_bytes!("vitamins.png");
    pub const DR_THROW: &[u8] = include_bytes!("dr/throw.png");
    pub const DR_IDLE: &[u8] = include_bytes!("dr/idle.png");
    pub const DR_GAME_OVER: &[u8] = include_bytes!("dr/game-over.png");
    pub const DR_VICTORY: &[u8] = include_bytes!("dr/victory.png");
    pub const BACKGROUND: &[u8] = include_bytes!("background.png");
    pub const BOTTLES: &[u8] = include_bytes!("bottles.png");
    pub const FONT_SMALL: &[u8] = include_bytes!("font_sm.png");
    pub const FONT_LARGE: &[u8] = include_bytes!("font_lg.png");
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
    pub const GAME_OVER: &[u8] = include_bytes!("game-over.ogg");
    pub const RECEIVE_GARBAGE: &[u8] = include_bytes!("garbage.ogg");
    pub const SPEED_LEVEL_UP: &[u8] = include_bytes!("speed-level-up.ogg");
    pub const DROP: &[u8] = include_bytes!("drop.ogg");
    pub const MOVE_PILL: &[u8] = include_bytes!("move.ogg");
    pub const PAUSE: &[u8] = include_bytes!("pause.ogg");
    pub const ROTATE: &[u8] = include_bytes!("rotate.ogg");
    pub const VICTORY_INTRO: &[u8] = include_bytes!("victory-intro.ogg");
    pub const VICTORY_REPEAT: &[u8] = include_bytes!("victory-repeat.ogg");
    pub const NEXT_LEVEL_JINGLE: &[u8] = include_bytes!("next-level-jingle.ogg");
}

const BLOCK_SIZE: u32 = 10;

fn block(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_SIZE as i32, j * BLOCK_SIZE as i32)
}

fn color(j: i32) -> BlockPoints {
    BlockPoints::new(
        [block(2, j), block(3, j)],
        [block(0, j), block(1, j)],
        [block(3, j), block(2, j)],
        [block(1, j), block(0, j)],
        block(4, j),
        vec![block(7, j), block(8, j), block(9, j), block(10, j)],
        vec![block(11, j), block(12, j)],
        vec![block(5, j), block(6, j)]
    )
}

fn match_end(i: i32, j: i32) -> Point {
    Point::new(i * 65 + 1, j * 129 + 1)
}

pub fn n64_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config
) -> Result<Theme<'a>, String> {
    let scene = SceneType::Tile { texture: sprites::BACKGROUND_TILE };

    let options = RetroThemeOptions {
        name: ThemeName::N64,
        scene_low: scene.clone(),
        scene_medium: scene.clone(),
        scene_high: scene,
        virus_animation_type: VirusAnimationType::YoYo,
        dr_victory_animation_type: DrAnimationType::N64_VICTORY,
        dr_game_over_animation_type: DrAnimationType::N64_GAME_OVER,
        sprites: VitaminSpriteSheetData::new(
            sprites::VITAMINS,
            pills(
                BLOCK_SIZE * 2, BLOCK_SIZE,
                block(13, 0), block(15, 0), block(17, 0),
                block(13, 1), block(15, 1), block(17, 1),
                block(13, 2), block(15, 2), block(17, 2)
            ),
            color(0),
            color(2),
            color(1),
            BLOCK_SIZE,
            0x50,
            DrSpriteSheetData::new(sprites::DR_THROW, 4),
            DrSpriteSheetData::new(sprites::DR_GAME_OVER, 21),
            DrSpriteSheetData::new(sprites::DR_VICTORY, 13),
            DrSpriteSheetData::new(sprites::DR_IDLE, 6),
        ),
        geometry: BottleGeometry::new(BLOCK_SIZE, 0, (8, 41)),
        audio: AudioTheme::new(
            config.audio, sound::MOVE_PILL, sound::ROTATE, sound::DROP,
            sound::DESTROY_VIRUS, sound::DESTROY_VIRUS_COMBO, sound::DESTROY_VITAMIN, sound::DESTROY_VITAMIN_COMBO,
            sound::PAUSE, sound::SPEED_LEVEL_UP, sound::RECEIVE_GARBAGE, sound::NEXT_LEVEL_JINGLE
        )?
            .with_game_music(sound::FEVER_INTRO, sound::FEVER_REPEAT)?
            .with_game_over_music(sound::GAME_OVER, None)?
            .with_next_level_music(sound::FEVER_NEXT_LEVEL, None)?
            .with_victory_music(sound::VICTORY_INTRO, sound::VICTORY_REPEAT)?,
        font: FontThemeOptions::new(
            vec![
                FontRenderOptions::numeric_sprites(sprites::FONT_SMALL, texture_creator, 1)?,
                FontRenderOptions::numeric_sprites(sprites::FONT_LARGE, texture_creator, 0)?
            ],
            ThemedNumeric::new(0, MetricSnips::zero_fill((111, 105), 9999999)),
            ThemedNumeric::new(1, MetricSnips::zero_fill((131, 143), 99)),
            ThemedNumeric::new(1, MetricSnips::zero_fill((131, 183), 99))
        ),
        bottles_file: sprites::BOTTLES,
        bottle_low: Point::new(0, 0),
        bottle_medium: Point::new(0, 0),
        bottle_high: Point::new(0, 0),
        bottle_width: 96,
        bottle_height: 209,
        background_file: sprites::BACKGROUND,
        bottle_point: Point::new(0, 0),
        match_end_file: sprites::MATCH_END,
        game_over_points: vec![Point::new(1, 1)],
        next_level_points: vec![Point::new(82, 1)],

        // TODO adjust
        dr_throw_point: Point::new(113, 6),
        dr_game_over_point: Point::new(110, 8),
        dr_victory_point: Point::new(113, 6),

        dr_order_first: true,
        dr_hand_point: Point::new(108, 39),
        hold_point: Point::new(155, 13),
        peek_point: Point::new(110, 55),
        peek_offset: 10,
        peek_max: 2,
        peek_scale: Some(0.82)
    };

    retro_theme(canvas, texture_creator, options)
}