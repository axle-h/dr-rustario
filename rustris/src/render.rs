use crate::game::tetromino::TetrominoShape;
use crate::game::Game;
use engine::game::geometry::Point;
use engine::game::GameEvent;
use engine::render::GameRender;

impl GameRender for Game {
    /// lines cleared minus one: single, double, triple, tetris
    fn clear_class(&self, event: &GameEvent) -> u16 {
        match event {
            GameEvent::Clear { count, .. } => count.saturating_sub(1).min(3) as u16,
            _ => 0,
        }
    }

    fn spawn_cells(&self) -> Vec<Point> {
        let shape = self
            .board()
            .tetromino()
            .map(|t| t.shape())
            .unwrap_or(TetrominoShape::I);
        let spawn = shape.meta().spawn_point();
        shape
            .meta()
            .rotated_minos(engine::game::geometry::Rotation::North)
            .into_iter()
            .map(|p| {
                let p = p + spawn;
                Point::new(p.x, crate::game::board::TOTAL_HEIGHT as i32 - 1 - p.y)
            })
            .collect()
    }
}
