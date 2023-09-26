use std::collections::HashMap;
use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::animate::dr::DrAnimationType;
use crate::animate::virus::VirusAnimationType;
use crate::config::Config;
use crate::game::MAX_SCORE;
use crate::game::random::MAX_VIRUSES;
use crate::game::rules::MAX_VIRUS_LEVEL;
use crate::theme::retro::{retro_theme, RetroThemeOptions};
use crate::theme::sprite_sheet::{BlockAnimations, BlockAnimationsData, BlockPoints, pills, VitaminSpriteSheetData};
use crate::theme::{Theme, ThemeName};
use crate::theme::animation::AnimationSpriteSheetData;
use crate::theme::font::{alpha_sprites, FontRenderOptions, FontThemeOptions, MetricSnips};
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
    pub const FONT: &[u8] = include_bytes!("font.png");
    pub const MATCH_END: &[u8] = include_bytes!("match-end.png");
}
mod sound {
    pub const FEVER_INTRO: &[u8] = include_bytes!("fever-intro.ogg");
    pub const FEVER_REPEAT: &[u8] = include_bytes!("fever-repeat.ogg");
    pub const FEVER_NEXT_LEVEL_INTRO: &[u8] = include_bytes!("fever-next-level-intro.ogg");
    pub const FEVER_NEXT_LEVEL_REPEAT: &[u8] = include_bytes!("fever-next-level-repeat.ogg");
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
    // const VIRUS_DEAD: &[u8] = include_bytes!("virus-dead.ogg");
    pub const VICTORY_INTRO: &[u8] = include_bytes!("victory-intro.ogg");
    pub const VICTORY_REPEAT: &[u8] = include_bytes!("victory-repeat.ogg");
    pub const NEXT_LEVEL_JINGLE: &[u8] = include_bytes!("next-level-jingle.ogg");
}

pub const BLOCK_SIZE: u32 = 7;

// 2 block wide + 2 outside borders + 1 inside border
const PILL_WIDTH: u32 = BLOCK_SIZE * 2 + 3;
// 1 block high + 2 outside borders
const PILL_HEIGHT: u32 = BLOCK_SIZE + 2;

fn block(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_SIZE as i32, j * BLOCK_SIZE as i32)
}

fn blocks(j: i32) -> BlockPoints {
    BlockPoints::new(
        [block(0, j), block(1, j)],
        [block(2, j), block(3, j)],
        [block(1, j), block(0, j)],
        [block(3, j), block(2, j)],
        block(4, j)
    )
}

fn animations(j: i32) -> BlockAnimationsData {
    BlockAnimationsData::non_exclusive_linear(
        sprites::VITAMINS,
        block(6, j), 2,
        block(5, j), 1,
        block(5, j), 1,
        BLOCK_SIZE
    )
}

fn pill(i: i32, j: i32) -> Point {
    Point::new(57 + i * 17, j * 9)
}

pub fn nes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config
) -> Result<Theme<'a>, String> {
    let options = RetroThemeOptions {
        name: ThemeName::Nes,
        scene_low: SceneType::Checkerboard { width: 8, height: 8, colors: [Color::BLACK, Color::RGB(0x00, 0x3f, 0x00)] },
        scene_medium: SceneType::Checkerboard { width: 8, height: 8, colors: [Color::BLACK, Color::RGB(0x2d, 0x05, 0x85)] },
        scene_high: SceneType::Checkerboard { width: 8, height: 8, colors: [Color::BLACK, Color::RGB(0x58, 0x58, 0x58)] },
        virus_animation_type: VirusAnimationType::LINEAR_STANDARD,
        dr_idle_animation_type: DrAnimationType::Static,
        dr_throw_animation_type: DrAnimationType::RETRO_THROW,
        dr_victory_animation_type: DrAnimationType::NES_SNES_VICTORY,
        dr_game_over_animation_type: DrAnimationType::Static,
        sprites: VitaminSpriteSheetData::new(
            sprites::VITAMINS,
            pills(
                    PILL_WIDTH, PILL_HEIGHT,
                pill(0, 0), pill(1, 0), pill(2, 0),
                pill(0, 1), pill(1, 1), pill(2, 1),
                pill(0, 2), pill(1, 2), pill(2, 2)
            ),
            (PILL_WIDTH, PILL_HEIGHT),
            blocks(0),
            animations(0),
            blocks(2),
            animations(2),
            blocks(1),
            animations(1),
            BLOCK_SIZE,
            0x40,
            AnimationSpriteSheetData::exclusive_linear(sprites::DR_THROW, 3),
            AnimationSpriteSheetData::exclusive_linear(sprites::DR_GAME_OVER, 1),
            AnimationSpriteSheetData::exclusive_linear(sprites::DR_VICTORY, 2),
            AnimationSpriteSheetData::exclusive_linear(sprites::DR_IDLE, 1),
            None
        ),
        geometry: BottleGeometry::new(7, 1, (8, 40)),
        audio: AudioTheme::new(
            config.audio, sound::MOVE_PILL, sound::ROTATE, sound::DROP,
            sound::DESTROY_VIRUS, sound::DESTROY_VIRUS_COMBO, sound::DESTROY_VITAMIN, sound::DESTROY_VITAMIN_COMBO,
            sound::PAUSE, sound::SPEED_LEVEL_UP, sound::RECEIVE_GARBAGE, sound::NEXT_LEVEL_JINGLE, None
            )?
            .with_game_music(sound::FEVER_INTRO, sound::FEVER_REPEAT)?
            .with_game_over_music(sound::GAME_OVER_INTRO, sound::GAME_OVER_REPEAT)?
            .with_next_level_music(sound::FEVER_NEXT_LEVEL_INTRO, sound::FEVER_NEXT_LEVEL_REPEAT)?
            .with_victory_music(sound::VICTORY_INTRO, sound::VICTORY_REPEAT)?,
        font: FontThemeOptions::simple(
            FontRenderOptions::numeric_sprites(sprites::FONT, texture_creator, 1)?,
            MetricSnips::zero_fill((92, 113), MAX_SCORE),
            MetricSnips::zero_fill((123, 134), MAX_VIRUS_LEVEL),
            MetricSnips::zero_fill((123, 155), MAX_VIRUSES)
        ),
        bottles_file: sprites::BOTTLES,
        bottle_low: Point::new(81, 0),
        bottle_medium: Point::new(0, 0),
        bottle_high: Point::new(162, 0),
        bottle_width: 80,
        bottle_height: 176,
        background_file: sprites::BACKGROUND,
        bottle_point: Point::new(0, 0),
        match_end_file: sprites::MATCH_END,
        game_over_points: vec![Point::new(65, 0), Point::new(65, 129)],
        next_level_points: vec![Point::new(0, 0), Point::new(0, 129)],
        dr_throw_point: Point::new(97, 37),
        dr_game_over_point: Point::new(97, 37),
        dr_victory_point: Point::new(102, 37),
        dr_order_first: false,
        dr_hand_point: Point::new(102, 30),
        hold_point: Point::new(125, 30),
        peek_point: Point::new(94, 55),
        peek_offset: 10,
        peek_max: 2,
        peek_scale: Some(0.75),
    };

    retro_theme(canvas, texture_creator, options)
}