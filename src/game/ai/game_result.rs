use std::ops::{Add, AddAssign, Div, Sub};
use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameResult {
    score: u32,
    lines: u32,
    level: u32,
    game_over: bool,
    time: Duration,
    pieces: u32,
    tetris_lines: u32,
}

impl Display for GameResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "score: {}, lines: {}, level: {}, pieces: {}, tetris lines: {}, game over: {}, time: {:?}",
               self.score, self.lines, self.level, self.pieces, self.tetris_lines, self.game_over, self.time)
    }
}

impl GameResult {
    pub fn new(score: u32, lines: u32, level: u32, game_over: bool, time: Duration) -> Self {
        Self { score, lines, level, game_over, time, pieces: 0, tetris_lines: 0 }
    }

    pub fn with_pieces(mut self, pieces: u32, tetris_lines: u32) -> Self {
        self.pieces = pieces;
        self.tetris_lines = tetris_lines;
        self
    }

    pub fn pieces(&self) -> u32 {
        self.pieces
    }

    /// lines that were cleared as part of a tetris (4 line clear)
    pub fn tetris_lines(&self) -> u32 {
        self.tetris_lines
    }

    /// fraction of cleared lines that were part of a tetris
    pub fn tetris_fraction(&self) -> f64 {
        if self.lines == 0 { 0.0 } else { self.tetris_lines as f64 / self.lines as f64 }
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn lines(&self) -> u32 {
        self.lines
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn game_over(&self) -> bool {
        self.game_over
    }

    pub fn time(&self) -> Duration {
        self.time
    }
}

impl Default for GameResult {
    fn default() -> Self {
        Self::new(0, 0, 0, false, Duration::ZERO)
    }
}

impl Add for GameResult {
    type Output = GameResult;

    fn add(self, rhs: Self) -> Self::Output {
        GameResult {
            score: self.score + rhs.score,
            lines: self.lines + rhs.lines,
            level: self.level + rhs.level,
            game_over: self.game_over || rhs.game_over,
            time: self.time + rhs.time,
            pieces: self.pieces + rhs.pieces,
            tetris_lines: self.tetris_lines + rhs.tetris_lines,
        }
    }
}

impl Sum for GameResult {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(GameResult::default(), |acc, r| acc + r)
    }
}

impl AddAssign for GameResult {
    fn add_assign(&mut self, rhs: Self) {
        self.score += rhs.score;
        self.lines += rhs.lines;
        self.level += rhs.level;
        self.game_over |= rhs.game_over;
        self.time += rhs.time;
        self.pieces += rhs.pieces;
        self.tetris_lines += rhs.tetris_lines;
    }
}

impl Sub for GameResult {
    type Output = f64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.score as f64 - rhs.score as f64
    }
}

impl Div<usize> for GameResult {
    type Output = GameResult;

    fn div(self, rhs: usize) -> Self::Output {
        let rhs_f64 = rhs as f64;
        GameResult {
            score: (self.score as f64 / rhs_f64).round() as u32,
            lines: (self.lines as f64 / rhs_f64).round() as u32,
            level: (self.level as f64 / rhs_f64).round() as u32,
            game_over: self.game_over,
            time: self.time.div_f64(rhs_f64),
            pieces: (self.pieces as f64 / rhs_f64).round() as u32,
            tetris_lines: (self.tetris_lines as f64 / rhs_f64).round() as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_results() {
        let a = GameResult::new(100, 10, 1, false, Duration::from_secs(1)).with_pieces(25, 8);
        let b = GameResult::new(300, 20, 2, true, Duration::from_secs(3)).with_pieces(50, 0);
        let avg = (a + b) / 2;
        assert_eq!(avg.score(), 200);
        assert_eq!(avg.lines(), 15);
        assert_eq!(avg.pieces(), 38);
        assert_eq!(avg.tetris_lines(), 4);
        assert!(avg.game_over());
    }

    #[test]
    fn tetris_fraction() {
        let r = GameResult::new(0, 10, 0, false, Duration::ZERO).with_pieces(0, 8);
        assert_eq!(r.tetris_fraction(), 0.8);
        assert_eq!(GameResult::default().tetris_fraction(), 0.0);
    }
}
