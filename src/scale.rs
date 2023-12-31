use sdl2::rect::{Point, Rect};
use std::cmp::min;

const PLAYER_BUFFER_PCT: f64 = 0.002;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scale {
    players: u32,
    scale: u32,
    window_width: u32,
    window_height: u32,
    block_size: u32,
    player_buffer_width: u32,
    player_buffer_height: u32,
}

impl Scale {
    pub fn new(
        players: u32,
        (bg_width, bg_height): (u32, u32),
        (window_width, window_height): (u32, u32),
        block_size: u32,
    ) -> Self {
        let player_buffer_width = (PLAYER_BUFFER_PCT * window_width as f64).round() as u32;
        let player_buffer_height = (PLAYER_BUFFER_PCT * window_height as f64).round() as u32;
        let effective_bg_width = bg_width + 2 * player_buffer_width;
        let effective_bg_height = bg_height + 2 * player_buffer_height;
        let scale = min(
            window_width / (effective_bg_width * players),
            window_height / effective_bg_height,
        );
        Self {
            players,
            scale,
            window_width,
            window_height,
            block_size: block_size * scale,
            player_buffer_width,
            player_buffer_height,
        }
    }

    /// splits the entire window up into horizontally stacked chunks equally between players
    pub fn player_window(&self, player: u32) -> Rect {
        let player_chunk_width = self.window_width / self.players;
        let x = player_chunk_width * player + self.player_buffer_width;
        Rect::new(
            x as i32,
            self.player_buffer_height as i32,
            player_chunk_width,
            self.window_height,
        )
    }

    pub fn scale_rect(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x * self.scale as i32,
            rect.y * self.scale as i32,
            rect.width() * self.scale,
            rect.height() * self.scale,
        )
    }

    pub fn scale_and_offset_rect(&self, rect: Rect, offset_x: i32, offset_y: i32) -> Rect {
        Rect::new(
            rect.x * self.scale as i32 + offset_x,
            rect.y * self.scale as i32 + offset_y,
            rect.width() * self.scale,
            rect.height() * self.scale,
        )
    }

    pub fn scale_and_offset_point(&self, point: Point, offset_x: i32, offset_y: i32) -> Point {
        Point::new(
            point.x * self.scale as i32 + offset_x,
            point.y * self.scale as i32 + offset_y,
        )
    }

    pub fn offset_proportional_to_block_size(
        &self,
        rect: Rect,
        offset_x: f64,
        offset_y: f64,
    ) -> Rect {
        let block_size = self.block_size as f64;
        Rect::new(
            (rect.x as f64 + offset_x * block_size).round() as i32,
            (rect.y as f64 + offset_y * block_size).round() as i32,
            rect.width(),
            rect.height(),
        )
    }

    pub fn window_size(&self) -> (u32, u32) {
        (self.window_width, self.window_height)
    }
}
