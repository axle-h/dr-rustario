use sdl2::image::LoadTexture;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::animate::dr::DrAnimationType;
use crate::animate::virus::VirusAnimationType;
use crate::game::pill::{left_vitamin_spawn_point, PillShape, Vitamin};
use crate::theme::{AnimationMeta, Theme, ThemeName};
use crate::theme::font::{FontRenderOptions, FontThemeOptions, MetricSnips};
use crate::theme::geometry::BottleGeometry;
use crate::theme::scene::SceneType;
use crate::theme::sound::AudioTheme;
use crate::theme::sprite_sheet::{DrType, VitaminSpriteSheet, VitaminSpriteSheetData};

pub struct RetroThemeOptions {
    pub name: ThemeName,
    pub scene_low: SceneType,
    pub scene_medium: SceneType,
    pub scene_high: SceneType,
    pub virus_animation_type: VirusAnimationType,
    pub dr_victory_animation_type: DrAnimationType,
    pub dr_game_over_animation_type: DrAnimationType,
    pub sprites: VitaminSpriteSheetData,
    pub geometry: BottleGeometry,
    pub audio: AudioTheme,
    pub font: FontThemeOptions,
    pub bottles_file: &'static [u8],
    pub bottle_low: Point,
    pub bottle_medium: Point,
    pub bottle_high: Point,
    pub bottle_width: u32,
    pub bottle_height: u32,
    pub background_file: &'static [u8],
    pub bottle_point: Point,
    pub dr_order_first: bool,
    pub dr_hand_point: Point,
    pub dr_throw_point: Point,
    pub dr_game_over_point: Point,
    pub dr_victory_point: Point,
    pub match_end_file: &'static [u8],
    pub game_over_points: Vec<Point>,
    pub next_level_points: Vec<Point>,
    pub hold_point: Point,
    pub peek_point: Point,
    pub peek_max: u32,
    pub peek_offset: i32,
    pub peek_scale: Option<f64>,
}

pub fn retro_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    options: RetroThemeOptions,
) -> Result<Theme<'a>, String> {
    let sprites = VitaminSpriteSheet::new(
        canvas,
        texture_creator,
        options.sprites,
        None,
    )?;
    let mut bottles_texture = texture_creator.load_texture_bytes(options.bottles_file)?;
    bottles_texture.set_blend_mode(BlendMode::Blend);

    let mut background_texture = texture_creator.load_texture_bytes(options.background_file)?;
    background_texture.set_blend_mode(BlendMode::Blend);
    let background_query = background_texture.query();

    let font = options.font.build(texture_creator)?;

    let mut match_end_texture = texture_creator.load_texture_bytes(options.match_end_file)?;
    match_end_texture.set_blend_mode(BlendMode::Blend);
    let game_over_snips: Vec<Rect> = options.game_over_points.iter().map(|p|
        Rect::new(p.x, p.y, options.geometry.width(), options.geometry.height())
    ).collect();
    let next_level_snips: Vec<Rect> = options.next_level_points.iter().map(|p|
        Rect::new(p.x, p.y, options.geometry.width(), options.geometry.height())
    ).collect();

    let animation_meta = AnimationMeta {
        virus_type: options.virus_animation_type,
        virus_frames: sprites.virus_frames(),
        vitamin_pop_frames: sprites.vitamin_pop_frames(),
        virus_pop_frames: sprites.virus_pop_frames(),
        throw_start: options.dr_hand_point,
        // we take 1 away from the throw end as thrown pills have a border
        throw_end: options.geometry.point(left_vitamin_spawn_point()) - Point::new(1, 1),
        dr_throw_frames: sprites.dr_frames(DrType::Throw),
        dr_victory_type: options.dr_victory_animation_type,
        dr_victory_frames: sprites.dr_frames(DrType::Victory),
        dr_idle_frames: sprites.dr_frames(DrType::Idle),
        dr_game_over_type: options.dr_game_over_animation_type,
        dr_game_over_frames: sprites.dr_frames(DrType::GameOver),
        game_over_screen_frames: game_over_snips.len(),
        next_level_interstitial_frames: next_level_snips.len(),
    };

    Ok(
        Theme {
            name: options.name,
            scene_low:  options.scene_low.build(canvas, texture_creator)?,
            scene_medium:  options.scene_medium.build(canvas, texture_creator)?,
            scene_high:  options.scene_high.build(canvas, texture_creator)?,
            sprites,
            geometry: options.geometry,
            audio: options.audio,
            font,
            bottles_texture,
            bottle_low_snip: Rect::new(
                options.bottle_low.x, options.bottle_low.y,
                options.bottle_width, options.bottle_height
            ),
            bottle_medium_snip: Rect::new(
                options.bottle_medium.x, options.bottle_medium.y,
                options.bottle_width, options.bottle_height
            ),
            bottle_high_snip: Rect::new(
                options.bottle_high.x, options.bottle_high.y,
                options.bottle_width, options.bottle_height
            ),
            bottle_bg_snip:  Rect::new(
                options.bottle_point.x(),
                options.bottle_point.y(),
                options.bottle_width,
                options.bottle_height
            ),
            background_texture,
            background_size: (background_query.width, background_query.height),
            dr_order_first: options.dr_order_first,
            dr_hand_point: options.dr_hand_point,
            dr_throw_point: options.dr_throw_point,
            dr_game_over_point: options.dr_game_over_point,
            dr_victory_point: options.dr_victory_point,
            animation_meta,
            game_over_snips,
            next_level_snips,
            match_end_texture,
            hold_point: options.hold_point,
            peek_point: options.peek_point,
            peek_offset: options.peek_offset,
            peek_scale: options.peek_scale,
            peek_max: options.peek_max,
        }
    )
}