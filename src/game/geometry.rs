use std::ops::{Add, AddAssign, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BottlePoint {
    x: i32,
    y: i32,
}

impl BottlePoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn translate_mut(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    pub const fn translate(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub const fn x(&self) -> i32 {
        self.x
    }

    pub const fn y(&self) -> i32 {
        self.y
    }
}

impl From<(i32, i32)> for BottlePoint {
    fn from((x, y): (i32, i32)) -> Self {
        BottlePoint::new(x, y)
    }
}

impl Neg for BottlePoint {
    type Output = BottlePoint;

    fn neg(self) -> Self::Output {
        BottlePoint::new(-self.x, -self.y)
    }
}

impl Sub for BottlePoint {
    type Output = BottlePoint;

    fn sub(self, rhs: Self) -> Self::Output {
        BottlePoint::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add for BottlePoint {
    type Output = BottlePoint;

    fn add(self, rhs: Self) -> Self::Output {
        self.translate(rhs.x, rhs.y)
    }
}

impl AddAssign for BottlePoint {
    fn add_assign(&mut self, rhs: Self) {
        self.translate_mut(rhs.x, rhs.y);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    North,
    East,
    South,
    West,
}
impl Rotation {
    pub fn rotate(&self, clockwise: bool) -> Rotation {
        match self {
            Rotation::North => {
                if clockwise {
                    Rotation::East
                } else {
                    Rotation::West
                }
            }
            Rotation::East => {
                if clockwise {
                    Rotation::South
                } else {
                    Rotation::North
                }
            }
            Rotation::South => {
                if clockwise {
                    Rotation::West
                } else {
                    Rotation::East
                }
            }
            Rotation::West => {
                if clockwise {
                    Rotation::North
                } else {
                    Rotation::South
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clockwise_rotation() {
        assert_eq!(Rotation::North.rotate(true), Rotation::East);
        assert_eq!(Rotation::West.rotate(true), Rotation::North);
    }

    #[test]
    fn anticlockwise_rotation() {
        assert_eq!(Rotation::North.rotate(false), Rotation::West);
        assert_eq!(Rotation::East.rotate(false), Rotation::North);
    }
}
