use crate::game::geometry::{BottlePoint, Rotation};

const SPAWN_POINT: BottlePoint = BottlePoint::new(3, -1);

const NORTH_PILLS: [BottlePoint; 2] = [BottlePoint::new(0, 1), BottlePoint::new(1, 1)];
const EAST_PILLS: [BottlePoint; 2] = [BottlePoint::new(0, 0), BottlePoint::new(0, 1)];
const WEST_PILLS: [BottlePoint; 2] = [BottlePoint::new(1, 1), BottlePoint::new(0, 1)];
const SOUTH_PILLS: [BottlePoint; 2] = [BottlePoint::new(0, 1), BottlePoint::new(0, 0)];

// https://tetris.wiki/Dr._Mario
const WALL_KICKS_H_TO_V: &[(i32, i32)] = &[(0, 0), (1, 0), (0, 1), (1, 1)];
const WALL_KICKS_V_TO_H: &[(i32, i32)] = &[(0, 0), (-1, 0)];

fn rotated_pills(rotation: Rotation) -> [BottlePoint; 2] {
    match rotation {
        Rotation::North => NORTH_PILLS,
        Rotation::East => EAST_PILLS,
        Rotation::South => WEST_PILLS,
        Rotation::West => SOUTH_PILLS,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VirusColor {
    Yellow = 0,
    Blue = 1,
    Red = 2
}

impl VirusColor {
    pub const N: usize = 3;

    pub fn next(self) -> Self {
        match self {
            VirusColor::Yellow => VirusColor::Blue,
            VirusColor::Blue => VirusColor::Red,
            VirusColor::Red => VirusColor::Yellow
        }
    }
}

impl TryFrom<usize> for VirusColor {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            x if x == VirusColor::Yellow as usize => Ok(VirusColor::Yellow),
            x if x == VirusColor::Red as usize => Ok(VirusColor::Red),
            x if x == VirusColor::Blue as usize => Ok(VirusColor::Blue),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PillShape {
    left_color: VirusColor,
    right_color: VirusColor
}

impl PillShape {
    pub const fn new(left_color: VirusColor, right_color: VirusColor) -> Self {
        Self { left_color, right_color }
    }

    pub const RED_RED: Self = Self::new(VirusColor::Red, VirusColor::Red);
    pub const RED_BLUE: Self = Self::new(VirusColor::Red, VirusColor::Blue);
    pub const BLUE_RED: Self = Self::new(VirusColor::Blue, VirusColor::Red);
    pub const BLUE_BLUE: Self = Self::new(VirusColor::Blue, VirusColor::Blue);

    pub fn left_color(&self) -> VirusColor {
        self.left_color
    }
    pub fn right_color(&self) -> VirusColor {
        self.right_color
    }
}

impl VirusColor {
    pub fn to_char(self) -> char {
        match self {
            VirusColor::Red => 'r',
            VirusColor::Blue => 'b',
            VirusColor::Yellow => 'y'
        }
    }
}

/// ordinal within a pill in the North rotation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VitaminOrdinal {
    Left = 0,
    Right = 1
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Garbage {
    color: VirusColor,
    position: BottlePoint
}

impl Garbage {
    pub fn new(color: VirusColor, position: BottlePoint) -> Self {
        Self { color, position }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vitamin {
    ordinal: VitaminOrdinal,
    color: VirusColor,
    position: BottlePoint
}

impl Vitamin {
    pub fn left(color: VirusColor, position: BottlePoint) -> Self {
        Self { ordinal: VitaminOrdinal::Left, color, position }
    }

    pub fn right(color: VirusColor, position: BottlePoint) -> Self {
        Self { ordinal: VitaminOrdinal::Right, color, position }
    }

    pub fn vitamins(shape: PillShape) -> Vitamins {
        [
            Vitamin::left(shape.left_color, NORTH_PILLS[0] + SPAWN_POINT),
            Vitamin::right(shape.right_color, NORTH_PILLS[1] + SPAWN_POINT)
        ]
    }

    pub fn ordinal(&self) -> VitaminOrdinal {
        self.ordinal
    }

    pub fn color(&self) -> VirusColor {
        self.color
    }

    pub fn position(&self) -> BottlePoint {
        self.position
    }

    pub fn translate(&mut self, dx: i32, dy: i32) {
        self.position.translate_mut(dx, dy);
    }

}

pub type Vitamins = [Vitamin; 2];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pill {
    vitamins: Vitamins,
    position: BottlePoint,
    rotation: Rotation,
    lock_placements: u32,
    y_min: i32,
}

impl Pill {
    pub fn new(shape: PillShape) -> Self {
        Self {
            vitamins: Vitamin::vitamins(shape),
            position: SPAWN_POINT,
            rotation: Rotation::North,
            lock_placements: 0,
            y_min: SPAWN_POINT.y()
        }
    }

    pub fn shape(&self) -> PillShape {
        PillShape::new(self.vitamins[0].color, self.vitamins[1].color)
    }

    pub fn vitamins(&self) -> Vitamins {
        self.vitamins
    }

    pub fn available_wall_kicks(&self) -> &[(i32, i32)] {
        match self.rotation {
            Rotation::North | Rotation::South => WALL_KICKS_H_TO_V,
            Rotation::East | Rotation::West => WALL_KICKS_V_TO_H
        }
    }

    pub fn next_rotation(&self, clockwise: bool) -> [BottlePoint; 2] {
        rotated_pills(self.rotation.rotate(clockwise))
            .map(|p| p + self.position)
    }

    pub fn rotate(&mut self, clockwise: bool, kick_x: i32, kick_y: i32) {
        self.rotation = self.rotation.rotate(clockwise);
        for (i, p) in rotated_pills(self.rotation).into_iter().enumerate() {
            self.vitamins[i].position = p + self.position;
        }
        self.translate(kick_x, kick_y);
    }

    pub fn translate(&mut self, dx: i32, dy: i32) {
        self.position.translate_mut(dx, dy);
        for vitamin in self.vitamins.iter_mut() {
            vitamin.position.translate_mut(dx, dy);
        }

        if self.position.y() < self.y_min {
            self.y_min = self.position.y();
            // lock placements are reset every time a pill falls
            self.lock_placements = 0;
        }
    }

    pub fn register_lock_placement(&mut self) -> u32 {
        self.lock_placements += 1;
        self.lock_placements
    }

    pub fn position(&self) -> BottlePoint {
        self.position
    }
    pub fn rotation(&self) -> Rotation {
        self.rotation
    }
    pub fn lock_placements(&self) -> u32 {
        self.lock_placements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_at_the_top_of_bottle() {
        let pill = Pill::new(PillShape::RED_BLUE);
        assert_eq!(pill.rotation, Rotation::North);
        assert_eq!(pill.position, SPAWN_POINT);
        assert_eq!(pill.lock_placements, 0);
        assert_eq!(pill.y_min, -1);

        assert_eq!(pill.vitamins, [
            Vitamin::left(VirusColor::Red, BottlePoint::new(3, 0)),
            Vitamin::right(VirusColor::Blue, BottlePoint::new(4, 0))
        ]);
    }

    #[test]
    fn rotates_clockwise() {
        let mut pill = Pill::new(PillShape::RED_BLUE);
        pill.rotate(true, 0, 1);
        assert_eq!(pill.rotation, Rotation::East);
        assert_eq!(pill.position, BottlePoint::new(3, 0)); // kicked down 1
        assert_eq!(pill.vitamins, [
            Vitamin::left(VirusColor::Red, BottlePoint::new(3, 0)),
            Vitamin::right(VirusColor::Blue, BottlePoint::new(3, 1))
        ]);
    }

    #[test]
    fn rotates_anticlockwise() {
        let mut pill = Pill::new(PillShape::RED_BLUE);
        pill.rotate(false, 0, 1);
        assert_eq!(pill.rotation, Rotation::West);
        assert_eq!(pill.position, BottlePoint::new(3, 0)); // kicked down 1
        assert_eq!(pill.vitamins, [
            Vitamin::left(VirusColor::Red, BottlePoint::new(3, 1)),
            Vitamin::right(VirusColor::Blue, BottlePoint::new(3, 0))
        ]);
    }

    #[test]
    fn color_try_into() {
        assert_eq!(VirusColor::Yellow, 0.try_into().unwrap());
        assert_eq!(VirusColor::Blue, 1.try_into().unwrap());
        assert_eq!(VirusColor::Red, 2.try_into().unwrap());
        let no_color: Result<VirusColor, ()> = 3.try_into();
        assert!(no_color.is_err());
    }
}