//! Builds a [`Theme`] from pre-drawn art: a background, board frame(s), a sprite sheet and
//! optional match-end overlays and mascot.

use crate::animate::destroy::DestroyStyle;
use crate::animate::frames::FrameAnimationType;
use crate::animate::game_over::GameOverStyle;
use crate::animate::mascot::MascotAnimationTypes;
use crate::animate::spawn::SpawnArc;
use crate::animate::AnimationMeta;
use crate::game::CellId;
use crate::render::font::FontThemeOptions;
use crate::render::geometry::BoardGeometry;
use crate::render::helper::{TextureFactory, TextureQuery};
use crate::render::scene::SceneType;
use crate::render::sound::AudioTheme;
use crate::render::sprite_sheet::{BlockSpriteSheet, BlockSpriteSheetData, GhostStyle, MascotKind};
use crate::render::{HoldLayout, MascotLayout, MatchEndSprites, PeekLayout, Theme};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

pub struct RetroThemeOptions {
    pub name: &'static str,
    /// one per speed band
    pub scenes: Vec<SceneType>,
    pub sprites: BlockSpriteSheetData,
    pub geometry: BoardGeometry,
    pub audio: AudioTheme,
    pub font: FontThemeOptions,
    pub board_file: &'static [u8],
    /// the board frame per speed band within `board_file`
    pub board_snips: Vec<Rect>,
    /// where the board frame sits in the background
    pub board_point: Point,
    pub background_file: &'static [u8],
    pub background_color: Color,
    /// full-board overlays: game over and stage-clear frames
    pub match_end_file: Option<&'static [u8]>,
    pub game_over_points: Vec<Point>,
    pub interstitial_points: Vec<Point>,
    pub hold: Option<HoldLayout>,
    pub peek: PeekLayout,
    pub mascot: Option<MascotLayout>,
    pub mascot_animations: Option<MascotAnimationTypes>,
    /// where a spawning piece is thrown from and to, in background coordinates
    pub spawn_arc: Option<(Point, Point)>,
    pub cell_idle_type: FrameAnimationType,
    /// defaults to popping each cell with its own strip
    pub destroy_style: Option<DestroyStyle>,
    /// defaults to the game over overlay frames
    pub game_over_style: Option<GameOverStyle>,
    pub curtain_cell: Option<CellId>,
    pub ghost_style: GhostStyle,
}

pub fn retro_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    options: RetroThemeOptions,
) -> Result<Theme<'a>, String> {
    let sprites = BlockSpriteSheet::new(canvas, texture_creator, &options.sprites, None)?;
    let board_texture = texture_creator.load_texture_bytes_blended(options.board_file)?;

    let background_texture =
        texture_creator.load_texture_bytes_blended(options.background_file)?;
    let background_size = background_texture.size();

    let font = options.font.build(texture_creator)?;

    let board_size = options
        .board_snips
        .first()
        .map(|r| (r.width(), r.height()))
        .unwrap_or((0, 0));

    let match_end = match options.match_end_file {
        Some(file) => {
            let overlay = |p: &Point| {
                Rect::new(
                    p.x,
                    p.y,
                    options.geometry.width(),
                    options.geometry.height(),
                )
            };
            Some(MatchEndSprites {
                texture: texture_creator.load_texture_bytes_blended(file)?,
                game_over_snips: options.game_over_points.iter().map(overlay).collect(),
                interstitial_snips: options.interstitial_points.iter().map(overlay).collect(),
            })
        }
        None => None,
    };

    let mascot = match (options.mascot_animations, sprites.mascot()) {
        (Some(types), Some(mascot)) => Some(types.with_frames(
            mascot.sheet(MascotKind::Idle).frame_count(),
            mascot.sheet(MascotKind::Spawn).frame_count(),
            mascot.sheet(MascotKind::Victory).frame_count(),
            mascot.sheet(MascotKind::GameOver).frame_count(),
        )),
        _ => None,
    };

    let animation_meta = AnimationMeta {
        destroy: options
            .destroy_style
            .unwrap_or_else(|| sprites.pop_style()),
        game_over: options.game_over_style.unwrap_or(GameOverStyle::Screen {
            frames: options.game_over_points.len().max(1),
        }),
        interstitial_frames: options.interstitial_points.len().max(1),
        cell_idle_type: options.cell_idle_type,
        cell_idle: sprites.idle_cells(),
        spawn_arc: options.spawn_arc.map(|(start, end)| SpawnArc {
            start,
            end,
            block_size: options.geometry.block_size(),
        }),
        mascot,
    };

    let mut scenes = vec![];
    for scene in options.scenes.iter() {
        scenes.push(scene.build(canvas, texture_creator)?);
    }
    assert!(!scenes.is_empty(), "a theme needs at least one scene");

    Ok(Theme {
        name: options.name,
        scenes,
        sprites,
        geometry: options.geometry,
        audio: options.audio,
        font,
        board_texture,
        board_snips: options.board_snips,
        background_texture,
        board_bg_snip: Rect::new(
            options.board_point.x(),
            options.board_point.y(),
            board_size.0,
            board_size.1,
        ),
        background_size,
        background_color: options.background_color,
        mascot: options.mascot,
        animation_meta,
        match_end,
        curtain_cell: options.curtain_cell,
        hold: options.hold,
        peek: options.peek,
        ghost_style: options.ghost_style,
        particle_color: None,
        integer_scale: false,
    })
}
