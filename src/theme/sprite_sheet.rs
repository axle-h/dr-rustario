use sdl2::image::LoadTexture;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::game::block::Block;
use crate::game::bottle::BOTTLE_HEIGHT;
use crate::game::Game;
use crate::game::geometry::Rotation;
use crate::game::pill::{VirusColor, VitaminOrdinal};
use crate::theme::geometry::BottleGeometry;

#[derive(Clone, Debug)]
pub struct BlockPoints {
    north: [Point; 2],
    east: [Point; 2],
    south: [Point; 2],
    west: [Point; 2],
    garbage: Point,
    virus_frames: Vec<Point>,
    virus_pop_frames: Vec<Point>,
    vitamin_pop_frames: Vec<Point>,
}

impl BlockPoints {
    pub fn new(
        north: [Point; 2], east: [Point; 2], south: [Point; 2], west: [Point; 2],
        garbage: Point,
        virus_frames: Vec<Point>, virus_pop_frames: Vec<Point>, vitamin_pop_frames: Vec<Point>
    ) -> Self {
        Self { north, east, south, west, garbage, virus_frames, virus_pop_frames, vitamin_pop_frames }
    }
}

#[derive(Clone, Debug)]
pub struct BlockSnips {
    color: VirusColor,
    width: u32,
    north: [Rect; 2],
    east: [Rect; 2],
    south: [Rect; 2],
    west: [Rect; 2],
    garbage: Rect,
    virus_frames: Vec<Rect>,
    virus_pop_frames: Vec<Rect>,
    vitamin_pop_frames: Vec<Rect>,
}

impl BlockSnips {
    fn flatten(&self) -> Vec<Rect> {
        self.north.into_iter()
            .chain(self.east)
            .chain(self.south)
            .chain(self.west)
            .chain([self.garbage])
            .chain(self.virus_frames.iter().copied())
            .chain(self.virus_pop_frames.iter().copied())
            .chain(self.vitamin_pop_frames.iter().copied())
            .collect()
    }

    fn vitamin(&self, rotation: Rotation, ordinal: VitaminOrdinal) -> Rect {
        let index = ordinal as usize;
        match rotation {
            Rotation::North => self.north[index],
            Rotation::East => self.east[index],
            Rotation::South => self.south[index],
            Rotation::West => self.west[index],
        }
    }
}

struct BlockContext {
    block_size: u32,
    x: i32,
    y: i32
}

impl BlockContext {
    pub fn new(y: i32, block_size: u32) -> Self {
        Self { block_size, x: 0, y }
    }

    fn width(&self) -> u32 {
        // x is post incremented so is the total width
        self.x as u32
    }

    fn next(&mut self) -> Rect {
        let result = Rect::new(self.x, self.y, self.block_size, self.block_size);
        self.x += self.block_size as i32;
        result
    }

    fn next2(&mut self) -> [Rect; 2] {
        [self.next(), self.next()]
    }

    fn next_vec(&mut self, n: usize) -> Vec<Rect> {
        (0..n).map(|_| self.next()).collect()
    }
}

pub struct VitaminSpriteSheetData {
    file: &'static [u8],
    yellow: BlockPoints,
    red: BlockPoints,
    blue: BlockPoints,
    source_block_size: u32,
    ghost_alpha: u8,
}

impl VitaminSpriteSheetData {
    pub fn new(
        file: &'static [u8],
        yellow: BlockPoints,
        red: BlockPoints,
        blue: BlockPoints,
        source_block_size: u32,
        ghost_alpha: u8 // TODO ghost
    ) -> Self {
        Self { file, yellow, red, blue, source_block_size, ghost_alpha }
    }

    fn points(&self, color: VirusColor) -> &BlockPoints {
        match color {
            VirusColor::Yellow => &self.yellow,
            VirusColor::Blue => &self.blue,
            VirusColor::Red => &self.red
        }
    }

    fn source_snips(&self, color: VirusColor) -> BlockSnips {
        let src = self.points(color);
        BlockSnips {
            color,
            north: src.north.map(|p| self.source_block(p)),
            east: src.east.map(|p| self.source_block(p)),
            south: src.south.map(|p| self.source_block(p)),
            west: src.west.map(|p| self.source_block(p)),
            garbage: self.source_block(src.garbage),
            virus_frames: src.virus_frames.iter().copied().map(|p| self.source_block(p)).collect(),
            virus_pop_frames: src.virus_pop_frames.iter().copied().map(|p| self.source_block(p)).collect(),
            vitamin_pop_frames: src.vitamin_pop_frames.iter().copied().map(|p| self.source_block(p)).collect(),
            width: 0, // doesnt matter, this is unused for source snips
        }
    }

    fn source_block(&self, p: Point) -> Rect {
        Rect::new(p.x, p.y, self.source_block_size, self.source_block_size)
    }

    fn target_snips(&self, color: VirusColor, y: i32, block_size: u32) -> BlockSnips {
        let src = self.points(color);
        let mut context = BlockContext::new(y, block_size);
        BlockSnips {
            color,
            north: context.next2(),
            east: context.next2(),
            south: context.next2(),
            west: context.next2(),
            garbage: context.next(),
            virus_frames: context.next_vec(src.virus_frames.len()),
            virus_pop_frames: context.next_vec(src.virus_pop_frames.len()),
            vitamin_pop_frames: context.next_vec(src.vitamin_pop_frames.len()),
            width: context.width()
        }
    }
}


fn scale_blocks<'a>(
    canvas: &mut WindowCanvas,
    data: &VitaminSpriteSheetData,
    src_texture: &Texture<'a>,
    target_texture: &mut Texture<'a>,
    target_snips: &BlockSnips,
) -> Result<(), String> {
    canvas.with_texture_canvas(target_texture, |c| {
        let src_snips = data.source_snips(target_snips.color);
        for (src, target) in src_snips.flatten().into_iter().zip(target_snips.flatten()) {
            c.copy(&src_texture, src, target).unwrap();
        }
    }).map_err(|e| e.to_string())
}

pub struct VitaminSpriteSheet<'a> {
    texture: Texture<'a>,
    yellow: BlockSnips,
    red: BlockSnips,
    blue: BlockSnips,
    // character: Texture<'a>,
    // character_rects: Vec<Rect>,
    block_size: u32,
}

impl<'a> VitaminSpriteSheet<'a> {
    pub fn draw_bottle(&self, canvas: &mut WindowCanvas, game: &Game, geometry: &BottleGeometry) -> Result<(), String> {
        for j in (0..BOTTLE_HEIGHT).rev() {
            for (i, block) in game.row(j).iter().copied().enumerate() {
                let dest = geometry.block_rect(i as u32, j);
                match block {
                    Block::Empty => {}
                    Block::Vitamin(color, rotation, ordinal) =>
                        self.draw_vitamin(canvas, color, rotation, ordinal, dest),
                    Block::Stack(color, rotation, ordinal) =>
                        self.draw_vitamin(canvas, color, rotation, ordinal, dest),
                    Block::Garbage(color) => canvas.copy(&self.texture, self.snips(color).garbage, dest),
                    Block::Virus(color) => {
                        // todo animate
                        canvas.copy(&self.texture, self.snips(color).virus_frames[0], dest)
                    }
                    Block::Ghost(_, _, _) => {}
                }
            }
        }
        Ok(())
    }

    fn draw_vitamin(
        &self,
        canvas: &mut WindowCanvas,
        color: VirusColor,
        rotation: Rotation,
        ordinal: VitaminOrdinal,
        dest: Rect
    ) -> Result<(), String> {
        let snip = self.snips(color).vitamin(rotation, ordinal);
        canvas.copy(&self.texture, snip, dest)
    }

    fn mino_rect(&self, point: Point) -> Rect {
        Rect::new(point.x(), point.y(), self.block_size, self.block_size)
    }

    fn snips(&self, color: VirusColor) -> &BlockSnips {
        match color {
            VirusColor::Yellow => &self.yellow,
            VirusColor::Blue => &self.blue,
            VirusColor::Red => &self.red
        }
    }
}

impl<'a> VitaminSpriteSheet<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        data: VitaminSpriteSheetData,
        block_size: u32
    ) -> Result<Self, String> {
        let sprite_src = texture_creator.load_texture_bytes(data.file)?;
        let yellow = data.target_snips(VirusColor::Yellow, 0, block_size);
        let red = data.target_snips(VirusColor::Red, block_size as i32, block_size);
        let blue = data.target_snips(VirusColor::Blue, 2 * block_size as i32, block_size);
        let width = yellow.width.max(red.width).max(blue.width);
        let height = block_size * 3;
        let mut texture = texture_creator
            .create_texture_target(None, width, height)
            .map_err(|e| e.to_string())?;

        scale_blocks(canvas, &data, &sprite_src, &mut texture, &yellow)?;
        scale_blocks(canvas, &data, &sprite_src, &mut texture, &red)?;
        scale_blocks(canvas, &data, &sprite_src, &mut texture, &blue)?;

        Ok(Self { texture, yellow, red, blue, block_size })
    }
}

