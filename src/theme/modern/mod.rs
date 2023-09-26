use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use game_metrics::{GameMetricsRow, GameMetricType};
use crate::animate::dr::DrAnimationType;
use crate::animate::virus::VirusAnimationType;
use crate::config::Config;
use crate::font::FontType;
use crate::game::bottle::BOTTLE_HEIGHT;
use crate::game::MAX_SCORE;
use crate::game::pill::left_vitamin_spawn_point;
use crate::game::random::MAX_VIRUSES;
use crate::game::rules::MAX_VIRUS_LEVEL;
use crate::theme::{AnimationMeta, Theme, ThemeName};
use crate::theme::animation::AnimationSpriteSheetData;
use crate::theme::font::{FontRender, FontTheme, MetricSnips, ThemedNumeric};
use crate::theme::geometry::BottleGeometry;
use crate::theme::helper::{TextureFactory, TextureQuery};
use crate::theme::modern::game_metrics::GameMetricsTable;
use crate::theme::scene::SceneType;
use crate::theme::sound::AudioTheme;
use crate::theme::sprite_sheet::{BlockAnimationsData, BlockPoints, DrType, pills, VitaminSpriteSheet, VitaminSpriteSheetData};

mod game_metrics;

pub mod sprites {
    // vitamins
    pub const VITAMINS: &[u8] = include_bytes!("vitamins-min.png");
    pub const SRC_BLOCK_SIZE: u32 = 100; // TODO I have this upto 400 does it look any nicer?

    // bottle
    pub const BOTTLE: &[u8] = include_bytes!("bottle.png");

    // viruses
    pub const VIRUS_RED_IDLE: &[u8] = include_bytes!("viruses/r-min.png");
    pub const VIRUS_BLUE_IDLE: &[u8] = include_bytes!("viruses/b-min.png");
    pub const VIRUS_YELLOW_IDLE: &[u8]= include_bytes!("viruses/y-min.png");

    // dr
    pub const DR_THROW: &[u8] = include_bytes!("dr/throw-min.png");
    pub const DR_IDLE: &[u8] = include_bytes!("dr/idle-min.png");
    pub const DR_GAME_OVER: &[u8] = include_bytes!("dr/game-over-min.png");
    pub const DR_VICTORY: &[u8] = include_bytes!("dr/victory-min.png");
    pub const SRC_DR_WIDTH: u32 = 478;
}

mod sound {
    pub const DESTROY_VIRUS: &[u8] = include_bytes!("destroy-virus.ogg");
    pub const DESTROY_VIRUS_COMBO: &[u8] = include_bytes!("destroy-virus-combo.ogg");
    pub const DESTROY_VITAMIN: &[u8] = include_bytes!("destroy-vitamin.ogg");
    pub const DESTROY_VITAMIN_COMBO: &[u8] = include_bytes!("destroy-vitamin-combo.ogg");
    pub const DROP: &[u8] = include_bytes!("drop.ogg");
    pub const FEVER_INTRO: &[u8] = include_bytes!("fever-intro.ogg");
    pub const FEVER_REPEAT: &[u8] = include_bytes!("fever-repeat.ogg");
    pub const FEVER_NEXT_LEVEL_INTRO: &[u8] = include_bytes!("fever-next-level-intro.ogg");
    pub const FEVER_NEXT_LEVEL_REPEAT: &[u8] = include_bytes!("fever-next-level-repeat.ogg");
    pub const GAME_OVER: &[u8] = include_bytes!("game-over.ogg");
    pub const RECEIVE_GARBAGE: &[u8] = include_bytes!("garbage.ogg");
    pub const HARD_DROP: &[u8] = include_bytes!("hard-drop.ogg");
    pub const MOVE_PILL: &[u8] = include_bytes!("move.ogg");
    pub const NEXT_LEVEL_JINGLE: &[u8] = include_bytes!("next-level-jingle.ogg");
    pub const PAUSE: &[u8] = include_bytes!("pause.ogg");
    pub const ROTATE: &[u8] = include_bytes!("rotate.ogg");
    pub const SPEED_LEVEL_UP: &[u8] = include_bytes!("speed-level-up.ogg");
    pub const VICTORY: &[u8] = include_bytes!("victory.ogg");
}

const BOTTLE_TOP_BUFFER_PCT: f64 = 0.15;
const MIN_VERTICAL_BUFFER_PCT: f64 = 0.02; // TODO this is actually useless and should be derived
const BOTTLE_BORDER_PCT_OF_BLOCK: f64 = 0.5;
const BOTTLE_BOARDER_SHADOW: u8 = 0x99;
const VERTICAL_GUTTER_PCT_OF_BLOCK: f64 = 0.2;

const DR_SCALE_OF_BLOCK: f64 = 6.5;

fn block(i: i32, j: i32) -> Point {
    Point::new(i * sprites::SRC_BLOCK_SIZE as i32, j * sprites::SRC_BLOCK_SIZE as i32)
}

fn pill(i: i32, j: i32) -> Point {
    Point::new(2 * i * sprites::SRC_BLOCK_SIZE as i32, j * sprites::SRC_BLOCK_SIZE as i32)
}

fn blocks(north_i: i32, east_j: i32, garbage_i: i32) -> BlockPoints {
    BlockPoints::new(
        [block(north_i, 0), block(north_i + 1, 0)],
        [block(6, east_j), block(6, east_j + 1)],
        [block(north_i + 1, 0), block(north_i, 0)],
        [block(6, east_j + 1), block(6, east_j)],
        block(garbage_i, 3)
    )
}

fn animations(virus_idle: &'static [u8], garbage_i: i32) -> BlockAnimationsData {
    BlockAnimationsData::new(
        AnimationSpriteSheetData::exclusive_square_linear(virus_idle),
        AnimationSpriteSheetData::static_first_square_frame(virus_idle),
        AnimationSpriteSheetData::non_exclusive_linear(
            sprites::VITAMINS,
            block(garbage_i, 3),
            1,
            sprites::SRC_BLOCK_SIZE,
            sprites::SRC_BLOCK_SIZE
        )
    )
}

pub fn modern_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    ttf: &Sdl2TtfContext,
    config: Config
) -> Result<Theme<'a>, String> {
    let (window_width, window_height) = canvas.window().size();
    dbg!(window_height);

    let bottle_top_buffer = (BOTTLE_TOP_BUFFER_PCT * window_height as f64).round() as u32;
    let block_size = (window_height as f64
        - (2.0 * window_height as f64 * MIN_VERTICAL_BUFFER_PCT)
        - bottle_top_buffer as f64)
        / BOTTLE_HEIGHT as f64;
    let border_weight = (block_size * BOTTLE_BORDER_PCT_OF_BLOCK).round() as u32;
    let vertical_gutter = (VERTICAL_GUTTER_PCT_OF_BLOCK * block_size).round() as u32;
    let block_size = block_size.round() as u32;

    let geometry = BottleGeometry::new(
        block_size,
        0,
        (border_weight as i32, bottle_top_buffer as i32)
    );

    let font_size = 2 * block_size / 3;
    let font = FontRender::from_font(
        canvas,
        texture_creator,
        ttf,
        FontType::Normal,
        font_size,
        Color::WHITE,
    )?;
    let font_bold = FontRender::from_font(
        canvas,
        texture_creator,
        ttf,
        FontType::Bold,
        font_size,
        Color::WHITE,
    )?;

    let bottle_snip = Rect::new(
        0,
        0,
        geometry.width() + 2 * border_weight,
        bottle_top_buffer + geometry.height() + border_weight
    );
    let mut metrics_right = GameMetricsTable::new(
        geometry.height() + bottle_top_buffer,
        &font,
        &font_bold,
        &[
            (GameMetricType::Score, MAX_SCORE),
            (GameMetricType::Level, MAX_VIRUS_LEVEL),
            (GameMetricType::VirusCount, MAX_VIRUSES),
        ],
    );
    metrics_right.offset_x(bottle_snip.right() + vertical_gutter as i32);

    let sprite_data = VitaminSpriteSheetData::new(
        sprites::VITAMINS,
        pills(
            sprites::SRC_BLOCK_SIZE * 2, sprites::SRC_BLOCK_SIZE,
            pill(2, 0), pill(2, 2), pill(2, 1),
            pill(0, 0), pill(0, 1), pill(0, 2),
            pill(1, 0), pill(1, 1), pill(1, 2)
        ),
        (block_size * 2, block_size),
        blocks(4, 4, 2),
        animations(sprites::VIRUS_YELLOW_IDLE, 2),
        blocks(2, 2, 1),
        animations(sprites::VIRUS_RED_IDLE, 1),
        blocks(0, 0, 0),
        animations(sprites::VIRUS_BLUE_IDLE, 0),
        sprites::SRC_BLOCK_SIZE,
        0x50,

        // all modern dr frames are 478 wide and all except victory are 478 high, victory is 510 high
        AnimationSpriteSheetData::exclusive_table(sprites::DR_THROW, 7, 7, 46),
        AnimationSpriteSheetData::exclusive_table(sprites::DR_GAME_OVER, 16, 15, 238),
        AnimationSpriteSheetData::exclusive_table(sprites::DR_VICTORY, 14, 14, 184),
        AnimationSpriteSheetData::exclusive_table(sprites::DR_IDLE, 12, 11, 123),
        Some(DR_SCALE_OF_BLOCK * block_size as f64 / sprites::SRC_DR_WIDTH as f64)
    );
    let sprites = VitaminSpriteSheet::new(
        canvas,
        texture_creator,
        sprite_data,
        block_size,
    )?;

    let dr_y = bottle_top_buffer as i32;
    let dr_x = bottle_snip.right() + vertical_gutter as i32;

    let (dr_throw_width, dr_throw_height) = sprites.dr_sprites(DrType::Throw).frame_size();
    let (dr_game_over_width, dr_game_over_height) = sprites.dr_sprites(DrType::GameOver).frame_size();
    let (dr_victory_width, dr_victory_height) = sprites.dr_sprites(DrType::Victory).frame_size();
    let dr_width = dr_throw_width.max(dr_game_over_width).max(dr_victory_width);
    let dr_height = dr_throw_height.max(dr_game_over_height).max(dr_victory_height);

    // HACK the dr hand point is empirical... how else would we find it?!
    let dr_hand_point = Point::new(dr_x, dr_y + 10 * dr_height as i32 / 19);
    let dr_throw_point = Point::new(dr_x, dr_y + (dr_height - dr_throw_height) as i32);
    let dr_game_over_point = Point::new(dr_x, dr_y + (dr_height - dr_game_over_height) as i32);
    let dr_victory_point = Point::new(dr_x, dr_y + (dr_height - dr_victory_height) as i32);

    let mut borders = vec![];
    let step = BOTTLE_BOARDER_SHADOW / border_weight as u8;
    for i in 0..border_weight {
        let j = border_weight - i - 1;
        let alpha = if j > 0 {
            BOTTLE_BOARDER_SHADOW - j as u8 * step
        } else {
            0xff
        };
        let rect = Rect::new(
            i as i32,
            bottle_top_buffer as i32,
            geometry.width() - 2 * i + 2 * border_weight,
            geometry.height() - i + border_weight,
        );
        borders.push((rect, alpha))
    }

    let all_metrics = metrics_right.rows();
    let mut bottle_texture = texture_creator.create_texture_target_blended(bottle_snip.width(), bottle_snip.height())?;
    canvas
        .with_texture_canvas(&mut bottle_texture, |c| {
            c.set_draw_color(Color::RGBA(0, 0, 0, 0));
            c.clear();
            for (r, color) in borders.iter().copied() {
                c.set_draw_color(Color::RGBA(color, color, color, color));
                c.draw_rect(r).unwrap();
            }
            // re-clear the board to get rid of the top of the border
            c.set_draw_color(Color::RGBA(0, 0, 0, 0));
            c.fill_rect(Rect::new(
                border_weight as i32,
                0,
                geometry.width(),
                bottle_top_buffer + geometry.height(),
            ))
                .unwrap();
        })
        .map_err(|e| e.to_string())?;

    dbg!(bottle_snip.width(), vertical_gutter, dr_width, metrics_right.width());
    let mut bg_texture = texture_creator.create_texture_target_blended(
        bottle_snip.width() + vertical_gutter + dr_width.max(metrics_right.width()),
        bottle_snip.height()
    )?;
    let background_size = bg_texture.size();
    dbg!(background_size);
    canvas
        .with_texture_canvas(&mut bg_texture, |c| {
            for row in all_metrics.iter() {
                font_bold
                    .render_string(c, row.label(), row.metric().label())
                    .unwrap();
            }
        })
        .map_err(|e| e.to_string())?;

    let dr_frame_time = Duration::from_secs(1) / 60;
    let animation_meta = AnimationMeta {
        virus_type: VirusAnimationType::Linear { fps: 30 },
        virus_frames: sprites.virus_frames(),
        vitamin_pop_frames: sprites.vitamin_pop_frames(),
        virus_pop_frames: sprites.virus_pop_frames(),
        throw_start: dr_hand_point,
        throw_end: geometry.point(left_vitamin_spawn_point()),
        dr_throw_type: DrAnimationType::Linear { duration: dr_frame_time },
        dr_throw_frames: sprites.dr_sprites(DrType::Throw).frame_count(),
        dr_victory_type: DrAnimationType::LinearWithPause {
            duration: dr_frame_time,
            pause_for: Duration::from_secs(3),
            resume_from_frame: 98
        },
        dr_victory_frames: sprites.dr_sprites(DrType::Victory).frame_count(),
        dr_idle_type: DrAnimationType::Linear { duration: dr_frame_time },
        dr_idle_frames: sprites.dr_sprites(DrType::Idle).frame_count(),
        dr_game_over_type: DrAnimationType::LinearWithPause {
            duration: dr_frame_time,
            pause_for: Duration::from_secs(3),
            resume_from_frame: 195
        },
        dr_game_over_frames: sprites.dr_sprites(DrType::GameOver).frame_count(),
        game_over_screen_frames: 1,
        next_level_interstitial_frames: 1,
    };

    let mut match_end_texture = texture_creator.create_texture_target_blended(geometry.width() * 2, geometry.height())?;
    let game_over_snip = Rect::new(0, 0, geometry.width(), geometry.height());
    let next_level_snip = Rect::new(geometry.width() as i32, 0, geometry.width(), geometry.height());
    canvas.with_texture_canvas(&mut match_end_texture, |c| {
        font.render_string_in_center(c, game_over_snip, "game over").unwrap();
        font.render_string_in_center(c, next_level_snip, "next level").unwrap();

    }).map_err(|e| e.to_string())?;

    let score_snips = all_metrics
        .iter()
        .find(|r| r.metric() == GameMetricType::Score)
        .unwrap()
        .value();
    let virus_level_snips = all_metrics
        .iter()
        .find(|r| r.metric() == GameMetricType::Level)
        .unwrap()
        .value();
    let virus_count_snips = all_metrics
        .iter()
        .find(|r| r.metric() == GameMetricType::VirusCount)
        .unwrap()
        .value();
    let font_theme = FontTheme::new(
        vec![font],
        ThemedNumeric::new(0, score_snips),
        ThemedNumeric::new(0, virus_level_snips),
        ThemedNumeric::new(0, virus_count_snips)
    );

    let audio = AudioTheme::new(
        config.audio, sound::MOVE_PILL, sound::ROTATE, sound::DROP,
        sound::DESTROY_VIRUS, sound::DESTROY_VIRUS_COMBO, sound::DESTROY_VITAMIN, sound::DESTROY_VITAMIN_COMBO,
        sound::PAUSE, sound::SPEED_LEVEL_UP, sound::RECEIVE_GARBAGE, sound::NEXT_LEVEL_JINGLE, sound::HARD_DROP
    )?
        .with_game_music(sound::FEVER_INTRO, sound::FEVER_REPEAT)?
        .with_game_over_music(sound::GAME_OVER, None)?
        .with_next_level_music(sound::FEVER_NEXT_LEVEL_INTRO, sound::FEVER_NEXT_LEVEL_REPEAT)?
        .with_victory_music(sound::VICTORY, None)?;

    let scene_type = SceneType::Particles { base_color: Color::WHITE };
    Ok(Theme {
        name: ThemeName::Modern,
        scene_low: scene_type.build(canvas, texture_creator)?,
        scene_medium: scene_type.build(canvas, texture_creator)?,
        scene_high: scene_type.build(canvas, texture_creator)?,
        sprites,
        geometry,
        audio,
        font: font_theme,
        bottles_texture: bottle_texture,
        bottle_low_snip: bottle_snip,
        bottle_medium_snip: bottle_snip,
        bottle_high_snip: bottle_snip,
        background_texture: bg_texture,
        bottle_bg_snip: bottle_snip,
        background_size,
        dr_order_first: true,
        dr_hand_point,
        dr_throw_point,
        dr_game_over_point,
        dr_victory_point,
        animation_meta,
        game_over_snips: vec![game_over_snip],
        next_level_snips: vec![next_level_snip],
        match_end_texture,
        hold_point: Point::new(dr_x + dr_throw_width as i32 - 2 * block_size as i32, bottle_top_buffer as i32),
        peek_point: dr_hand_point + Point::new(0, (1.5 * block_size as f64).round() as i32),
        peek_offset: block_size as i32,
        peek_max: 2,
        peek_scale: Some(0.7)
    })
}
