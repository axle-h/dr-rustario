use crate::animate::dr::DrAnimationType;
use crate::animate::virus::VirusAnimationType;
use crate::config::Config;
use crate::game::random::MAX_VIRUSES;
use crate::game::rules::MAX_VIRUS_LEVEL;
use crate::game::MAX_SCORE;
use crate::theme::animation::AnimationSpriteSheetData;
use crate::theme::font::{FontRenderOptions, FontThemeOptions, MetricSnips};
use crate::theme::geometry::BottleGeometry;
use crate::theme::retro::{retro_theme, RetroThemeOptions};
use crate::theme::scene::SceneType;
use crate::theme::sound::AudioTheme;
use crate::theme::sprite_sheet::{pills, BlockAnimationsData, BlockPoints, VitaminSpriteSheetData};
use crate::theme::{Theme, ThemeName};

use sdl2::rect::Point;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

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
    pub const NEXT_LEVEL_JINGLE: &[u8] = include_bytes!("next-level-jingle.ogg");
}

pub const BLOCK_SIZE: u32 = 8;

fn block(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_SIZE as i32, j * BLOCK_SIZE as i32)
}

fn blocks(j: i32) -> BlockPoints {
    BlockPoints::new(
        [block(4, j), block(5, j)],
        [block(11, j), block(10, j)],
        [block(5, j), block(4, j)],
        [block(10, j), block(11, j)],
        block(3, j),
    )
}

fn animations(j: i32) -> BlockAnimationsData {
    BlockAnimationsData::non_exclusive_linear(
        sprites::VITAMINS,
        block(0, j),
        2,
        block(2, j),
        1,
        block(2, j),
        1,
        BLOCK_SIZE,
    )
}

fn match_end(i: i32, j: i32) -> Point {
    Point::new(i * 65 + 1, j * 129 + 1)
}

pub fn snes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config,
) -> Result<Theme<'a>, String> {
    let scene = SceneType::Tile {
        texture: sprites::BACKGROUND_TILE,
    };

    let options = RetroThemeOptions {
        name: ThemeName::Snes,
        scene_low: scene.clone(),
        scene_medium: scene.clone(),
        scene_high: scene,
        virus_animation_type: VirusAnimationType::LINEAR_STANDARD,
        dr_idle_animation_type: DrAnimationType::Static,
        dr_throw_animation_type: DrAnimationType::RETRO_THROW,
        dr_victory_animation_type: DrAnimationType::NES_SNES_VICTORY,
        dr_game_over_animation_type: DrAnimationType::Static,
        sprites: VitaminSpriteSheetData::new(
            sprites::VITAMINS,
            pills(
                BLOCK_SIZE * 2,
                BLOCK_SIZE,
                block(4, 0),
                block(6, 0),
                block(8, 0),
                block(4, 1),
                block(6, 1),
                block(8, 1),
                block(4, 2),
                block(6, 2),
                block(8, 2),
            ),
            (BLOCK_SIZE * 2, BLOCK_SIZE),
            blocks(0),
            animations(0),
            blocks(2),
            animations(2),
            blocks(1),
            animations(1),
            BLOCK_SIZE,
            0x50,
            AnimationSpriteSheetData::exclusive_linear(sprites::DR_THROW, 3),
            AnimationSpriteSheetData::exclusive_linear(sprites::DR_GAME_OVER, 1),
            AnimationSpriteSheetData::exclusive_linear(sprites::DR_VICTORY, 2),
            AnimationSpriteSheetData::exclusive_linear(sprites::DR_IDLE, 1),
            None,
        ),
        geometry: BottleGeometry::new(BLOCK_SIZE, 0, (7, 39)),
        audio: AudioTheme::new(
            config.audio,
            sound::MOVE_PILL,
            sound::ROTATE,
            sound::DROP,
            sound::DESTROY_VIRUS,
            sound::DESTROY_VIRUS_COMBO,
            sound::DESTROY_VITAMIN,
            sound::DESTROY_VITAMIN_COMBO,
            sound::PAUSE,
            sound::SPEED_LEVEL_UP,
            sound::RECEIVE_GARBAGE,
            sound::NEXT_LEVEL_JINGLE,
            None,
        )?
        .with_game_music(sound::FEVER_INTRO, sound::FEVER_REPEAT)?
        .with_game_over_music(sound::GAME_OVER_INTRO, sound::GAME_OVER_REPEAT)?
        .with_next_level_music(sound::FEVER_NEXT_LEVEL, None)?
        .with_victory_music(sound::VICTORY_INTRO, sound::VICTORY_REPEAT)?,
        font: FontThemeOptions::simple(
            FontRenderOptions::numeric_sprites(sprites::FONT, texture_creator, 1)?,
            MetricSnips::zero_fill((91, 110), MAX_SCORE),
            MetricSnips::zero_fill((123, 131), MAX_VIRUS_LEVEL),
            MetricSnips::zero_fill((123, 152), MAX_VIRUSES),
        ),
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
            match_end(2, 0),
            match_end(3, 0),
            match_end(4, 0),
            match_end(0, 1),
            match_end(1, 1),
            match_end(2, 1),
            match_end(3, 1),
            match_end(4, 1),
        ],
        dr_throw_end_offset: Point::new(0, 0),
        dr_throw_point: Point::new(99, 29),
        dr_game_over_point: Point::new(100, 31),
        dr_victory_point: Point::new(105, 31),
        dr_order_first: false,
        dr_hand_point: Point::new(103, 22),
        hold_point: Point::new(125, 18),
        peek_point: Point::new(96, 46),
        peek_offset: 10,
        peek_max: 2,
        peek_scale: Some(0.82),
    };

    retro_theme(canvas, texture_creator, options)
}
