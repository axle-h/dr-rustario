//! Everything drawn for one game on one theme. A [`Theme`] is data assembled by a builder
//! ([`retro::retro_theme`] or [`modern::modern_theme`]) from what a game crate declares.

pub mod animation;
pub mod block_mask;
pub mod context;
pub mod font;
pub mod geometry;
pub mod helper;
pub mod metrics_table;
pub mod modern;
pub mod pause;
pub mod retro;
pub mod scene;
pub mod sound;
pub mod sprite_sheet;

use crate::animate::game_over::GameOverStyle;
use crate::animate::{AnimationMeta, PlayerAnimations};
use crate::game::{CellId, Game, GameEvent, PieceId};
use crate::particles::particle::ParticleAnimationType;
use crate::particles::prescribed::RaceTheme;
use crate::render::font::FontTheme;
use crate::render::geometry::BoardGeometry;
use crate::render::scene::SceneRender;
use crate::render::sound::AudioTheme;
use crate::render::sprite_sheet::{BlockSpriteSheet, GhostStyle, MascotKind};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};

/// What a game tells the renderer beyond its board: how to grade events for sound and
/// particles. Everything visual comes from theme data.
pub trait GameRender {
    /// grade a [`GameEvent::Clear`] for [`sound::SfxKey::Clear`]
    fn clear_class(&self, event: &GameEvent) -> u16 {
        let _ = event;
        0
    }

    /// the cells a freshly spawned piece occupies, for spawn particles
    fn spawn_cells(&self) -> Vec<crate::game::geometry::Point>;
}

/// Where the mascot sits and the piece it holds.
#[derive(Clone, Copy, Debug)]
pub struct MascotLayout {
    /// where the next piece waits in the mascot's hand
    pub hand_point: Point,
    pub spawn_point: Point,
    pub game_over_point: Point,
    pub victory_point: Point,
    /// draw the mascot before the piece in its hand (so the piece overlaps it)
    pub draw_first: bool,
}

/// Where queued pieces are drawn.
#[derive(Clone, Debug)]
pub enum PeekLayout {
    /// a column of pieces starting at `point`, each `offset` further down. With a mascot the
    /// first queued piece is in its hand and the column shows the rest.
    Column {
        point: Point,
        offset: i32,
        max: u32,
        scale: Option<f64>,
    },
    /// explicit slots, each filled by one piece scaled to fit
    Slots { slots: Vec<Rect>, max_scale: f64 },
}

#[derive(Clone, Debug)]
pub enum HoldLayout {
    Point { point: Point, scale: Option<f64> },
    Slot { slot: Rect, max_scale: f64 },
}

/// Full-board overlays for the end of a match or stage.
pub struct MatchEndSprites<'a> {
    pub texture: Texture<'a>,
    pub game_over_snips: Vec<Rect>,
    pub interstitial_snips: Vec<Rect>,
}

pub struct Theme<'a> {
    pub(crate) name: &'static str,
    pub(crate) scenes: Vec<SceneRender<'a>>,
    pub(crate) sprites: BlockSpriteSheet<'a>,
    pub(crate) geometry: BoardGeometry,
    pub(crate) audio: AudioTheme,
    pub(crate) font: FontTheme<'a>,
    /// the board frame per speed band, drawn under the cells
    pub(crate) board_texture: Texture<'a>,
    pub(crate) board_snips: Vec<Rect>,
    pub(crate) background_texture: Texture<'a>,
    /// where the board texture sits within the background
    pub(crate) board_bg_snip: Rect,
    pub(crate) background_size: (u32, u32),
    pub(crate) background_color: Color,
    pub(crate) mascot: Option<MascotLayout>,
    pub(crate) animation_meta: AnimationMeta,
    pub(crate) match_end: Option<MatchEndSprites<'a>>,
    /// the cell drawn by a curtain game over
    pub(crate) curtain_cell: Option<CellId>,
    pub(crate) hold: Option<HoldLayout>,
    pub(crate) peek: PeekLayout,
    pub(crate) ghost_style: GhostStyle,
    /// themes that emit particles do so in this colour
    pub(crate) particle_color: Option<Color>,
    /// the window-scaling rule: true for themes that size themselves to the window
    pub(crate) integer_scale: bool,
}

impl<'a> Theme<'a> {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn sprites(&self) -> &BlockSpriteSheet<'a> {
        &self.sprites
    }

    fn band(&self, speed_index: u32) -> usize {
        (speed_index as usize).min(self.scenes.len().saturating_sub(1))
    }

    pub fn scene(&self, speed_index: u32) -> &SceneRender<'a> {
        &self.scenes[self.band(speed_index)]
    }

    pub fn animation_meta(&self) -> &AnimationMeta {
        &self.animation_meta
    }

    pub fn geometry(&self) -> &BoardGeometry {
        &self.geometry
    }

    pub fn background_size(&self) -> (u32, u32) {
        self.background_size
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn board_snip(&self) -> Rect {
        self.board_bg_snip
    }

    pub fn audio(&self) -> &AudioTheme {
        &self.audio
    }

    pub fn particle_color(&self) -> Option<Color> {
        self.particle_color
    }

    pub fn is_integer_scale(&self) -> bool {
        self.integer_scale
    }

    /// what this theme contributes to the menu's piece race, see
    /// [`crate::particles::prescribed::prescribed_piece_race`]
    pub fn race_theme(&self, index: usize, pieces: Vec<PieceId>, scale: f64) -> RaceTheme {
        let meta = &self.animation_meta;
        RaceTheme {
            theme: index,
            pieces,
            cells: meta
                .cell_idle
                .iter()
                .map(|(id, frames)| {
                    (
                        *id,
                        ParticleAnimationType::from_frames(meta.cell_idle_type, *frames),
                    )
                })
                .collect(),
            mascot: meta
                .mascot
                .map(|m| ParticleAnimationType::from_frames(m.idle_type, m.idle_frames)),
            scale,
        }
    }

    fn draw_mascot(
        &self,
        canvas: &mut WindowCanvas,
        kind: MascotKind,
        frame: Option<usize>,
    ) -> Result<(), String> {
        let Some(layout) = self.mascot else {
            return Ok(());
        };
        let point = match kind {
            MascotKind::Idle | MascotKind::Spawn => layout.spawn_point,
            MascotKind::GameOver => layout.game_over_point,
            MascotKind::Victory => layout.victory_point,
        };
        self.sprites
            .draw_mascot(canvas, kind, point, frame.unwrap_or(0))
    }

    fn draw_hold(&self, canvas: &mut WindowCanvas, piece: PieceId) -> Result<(), String> {
        match &self.hold {
            Some(HoldLayout::Point { point, scale }) => {
                self.sprites
                    .previews()
                    .draw_piece(canvas, piece, *point, None, *scale)
            }
            Some(HoldLayout::Slot { slot, max_scale }) => self
                .sprites
                .previews()
                .draw_piece_fill(canvas, piece, *slot, *max_scale),
            None => Ok(()),
        }
    }

    fn draw_queue(
        &self,
        canvas: &mut WindowCanvas,
        queue: &[PieceId],
        spawn_peek_offset: Option<f64>,
    ) -> Result<(), String> {
        match &self.peek {
            PeekLayout::Column {
                point,
                offset,
                max,
                scale,
            } => {
                let skip = if self.mascot.is_some() { 1 } else { 0 };
                let shift = spawn_peek_offset
                    .map(|o| *offset - (o * *offset as f64).round() as i32)
                    .unwrap_or(0);
                for (i, piece) in queue.iter().skip(skip).take(*max as usize).enumerate() {
                    let dest = point.offset(0, shift + i as i32 * *offset);
                    self.sprites
                        .previews()
                        .draw_piece(canvas, *piece, dest, None, *scale)?;
                }
                Ok(())
            }
            PeekLayout::Slots { slots, max_scale } => {
                for (slot, piece) in slots.iter().zip(queue.iter()) {
                    self.sprites
                        .previews()
                        .draw_piece_fill(canvas, *piece, *slot, *max_scale)?;
                }
                Ok(())
            }
        }
    }

    pub fn draw_background<G: Game>(
        &self,
        canvas: &mut WindowCanvas,
        game: &G,
        animations: &PlayerAnimations,
    ) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();
        let (width, height) = self.background_size;
        canvas.copy(
            &self.background_texture,
            None,
            Rect::new(0, 0, width, height),
        )?;

        let queue = game.queue();
        let mut spawn_peek_offset = None;
        let mut draw_previews = true;

        if let Some(game_over) = animations.game_over().state() {
            if self.mascot.is_some() {
                self.draw_mascot(canvas, MascotKind::GameOver, game_over.mascot_frame())?;
                draw_previews = false;
            }
        } else if let Some(victory) = animations.victory().state() {
            if self.mascot.is_some() {
                self.draw_mascot(canvas, MascotKind::Victory, victory.mascot_frame())?;
                draw_previews = false;
            }
        } else if let Some(interstitial) = animations.interstitial().state() {
            if self.mascot.is_some() {
                self.draw_mascot(canvas, MascotKind::Victory, interstitial.mascot_frame())?;
                draw_previews = false;
            }
        } else if let Some(spawn) = animations.spawn().state() {
            spawn_peek_offset = spawn.peek_offset();
            if let Some(layout) = self.mascot {
                let draw_piece = |canvas: &mut WindowCanvas| {
                    self.sprites.previews().draw_piece(
                        canvas,
                        spawn.piece(),
                        spawn.throw_position(),
                        spawn.piece_rotate_angle_degrees(),
                        None,
                    )
                };
                if layout.draw_first {
                    self.draw_mascot(canvas, MascotKind::Spawn, spawn.mascot_frame())?;
                    draw_piece(canvas)?;
                } else {
                    draw_piece(canvas)?;
                    self.draw_mascot(canvas, MascotKind::Spawn, spawn.mascot_frame())?;
                }
            }
        } else if let Some(layout) = self.mascot {
            let frame = animations.mascot_idle_frame();
            let draw_hand = |canvas: &mut WindowCanvas| match queue.first() {
                Some(piece) => self.sprites.previews().draw_piece(
                    canvas,
                    *piece,
                    layout.hand_point,
                    None,
                    None,
                ),
                None => Ok(()),
            };
            if layout.draw_first {
                self.draw_mascot(canvas, MascotKind::Idle, frame)?;
                draw_hand(canvas)?;
            } else {
                draw_hand(canvas)?;
                self.draw_mascot(canvas, MascotKind::Idle, frame)?;
            }
        }

        if draw_previews {
            if let Some(hold) = game.held() {
                self.draw_hold(canvas, hold)?;
            }
            self.draw_queue(canvas, &queue, spawn_peek_offset)?;
        }

        self.font.render_all(canvas, game)
    }

    pub fn draw_board<G: Game>(
        &self,
        canvas: &mut WindowCanvas,
        game: &G,
        animations: &PlayerAnimations,
    ) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        let board_snip = self.board_snips[self.band(game.speed_index())];
        let board_dest = Rect::new(0, 0, board_snip.width(), board_snip.height());
        canvas.copy(&self.board_texture, board_snip, board_dest)?;

        self.sprites
            .draw_board(canvas, game, &self.geometry, animations, self.ghost_style)?;

        if let Some(rows) = animations.game_over().curtain_rows() {
            if let Some(cell) = self.curtain_cell {
                for j in rows {
                    for i in 0..self.geometry.columns() {
                        let point = crate::game::geometry::Point::from_u32(
                            i,
                            j + self.geometry.hidden_rows(),
                        );
                        self.sprites.draw_cell(
                            canvas,
                            cell,
                            true,
                            self.geometry.raw_block(point),
                            0.0,
                            None,
                        )?;
                    }
                }
            }
        }

        if let Some(match_end) = &self.match_end {
            if let Some(frame) = animations
                .game_over()
                .state()
                .and_then(|s| s.screen_frame())
            {
                if let GameOverStyle::Screen { .. } = animations.game_over().style() {
                    if let Some(snip) = match_end.game_over_snips.get(frame) {
                        canvas.copy(&match_end.texture, *snip, self.geometry.game_snip())?;
                    }
                }
            } else if let Some(frame) = animations
                .interstitial()
                .state()
                .map(|s| s.interstitial_frame())
            {
                if let Some(snip) = match_end.interstitial_snips.get(frame) {
                    canvas.copy(&match_end.texture, *snip, self.geometry.game_snip())?;
                }
            }
        }

        Ok(())
    }
}
