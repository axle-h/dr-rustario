use std::collections::{HashMap, HashSet};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::animate::PlayerAnimations;
use crate::game::block::Block;
use crate::game::bottle::BOTTLE_HEIGHT;
use crate::game::Game;
use crate::game::geometry::{BottlePoint, Rotation};
use crate::game::pill::{PillShape, VirusColor, VitaminOrdinal};
use crate::theme::geometry::BottleGeometry;

const ALPHA_STRIDE: u8 = 4;

fn alpha_stride(alpha_mod: u8) -> u8 {
    ALPHA_STRIDE * (alpha_mod as f64 / ALPHA_STRIDE as f64).round() as u8
}

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
    height: u32,
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
        [self.garbage].into_iter()
            .chain(self.north)
            .chain(self.east)
            .chain(self.south)
            .chain(self.west)
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
    height: u32,
    x: i32,
    y: i32
}

impl BlockContext {
    pub fn new(y: i32, block_size: u32) -> Self {
        Self { block_size, x: 0, y, height: 0 }
    }

    fn width(&self) -> u32 {
        // x is post incremented so is the total width
        self.x as u32
    }

    fn next(&mut self) -> Rect {
        let result = Rect::new(self.x, self.y, self.block_size, self.block_size);
        self.x += self.block_size as i32;
        self.height = self.height.max(self.block_size);
        result
    }

    fn next2(&mut self) -> [Rect; 2] {
        [self.next(), self.next()]
    }

    fn next_vec(&mut self, n: usize) -> Vec<Rect> {
        (0..n).map(|_| self.next()).collect()
    }

    fn next_unscaled(&mut self, width: u32, height: u32) -> Rect {
        let result = Rect::new(self.x, self.y, width, height);
        self.x += width as i32;
        self.height = self.height.max(height);
        result
    }
}

fn pill_size(block_size: u32) -> (u32, u32) {
    (
        // 2 block wide + 2 outside borders + 1 inside border
        block_size * 2 + 3,
        // 1 block high + 2 outside borders
        block_size + 2
    )
}

fn pill_rect(point: Point, block_size: u32) -> Rect {
    let (width, height) = pill_size(block_size);
    Rect::new(point.x, point.y, width, height)
}

#[derive(Clone, Debug)]
pub struct PillSnips {
    width: u32,
    height: u32,
    shapes: HashMap<PillShape, Rect>
}

impl PillSnips {
    fn snip(&self, shape: PillShape) -> Rect {
        *self.shapes.get(&shape).unwrap()
    }
}

pub fn pills(
    yy: Point, yb: Point, yr: Point,
    bb: Point, by: Point, br: Point,
    rr: Point, ry: Point, rb: Point
) -> HashMap<PillShape, Point> {
    HashMap::from_iter([
        (PillShape::YY, yy), (PillShape::YB, yb), (PillShape::YR, yr),
        (PillShape::BB, bb), (PillShape::BY, by), (PillShape::BR, br),
        (PillShape::RR, rr), (PillShape::RY, ry), (PillShape::RB, rb),
    ])
}

pub struct VitaminSpriteSheetData {
    file: &'static [u8],
    pills: HashMap<PillShape, Point>,
    yellow: BlockPoints,
    red: BlockPoints,
    blue: BlockPoints,
    source_block_size: u32,
    ghost_alpha: u8,
    dr_normal: Vec<Point>,
    dr_normal_size: (u32, u32),
    dr_game_over: Vec<Point>,
    dr_game_over_size: (u32, u32),
    dr_victory: Vec<Point>,
    dr_victory_size: (u32, u32),
    dr_wait: Vec<Point>,
    dr_wait_size: (u32, u32),
}

impl VitaminSpriteSheetData {
    pub fn new(
        file: &'static [u8],
        pills: HashMap<PillShape, Point>,
        yellow: BlockPoints,
        red: BlockPoints,
        blue: BlockPoints,
        source_block_size: u32,
        ghost_alpha: u8,
        dr_normal: Vec<Point>,
        dr_normal_size: (u32, u32),
        dr_game_over: Vec<Point>,
        dr_game_over_size: (u32, u32),
        dr_victory: Vec<Point>,
        dr_victory_size: (u32, u32),
        dr_wait: Vec<Point>,
        dr_wait_size: (u32, u32),
    ) -> Self {
        Self {
            file,
            pills,
            yellow,
            red,
            blue,
            source_block_size,
            ghost_alpha,
            dr_normal,
            dr_normal_size,
            dr_game_over,
            dr_game_over_size,
            dr_victory,
            dr_victory_size,
            dr_wait,
            dr_wait_size
        }
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
            width: 0, // doesnt matter, these are unused for source snips
            height: 0
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
            width: context.width(),
            height: context.height
        }
    }

    fn pill_source_snips(&self, source_block_size: u32) -> PillSnips {
        PillSnips {
            shapes: self.pills.iter()
                .map(|(s, p)| (*s, pill_rect(*p, source_block_size)))
                .collect(),
            // width and height are not used
            width: 0,
            height: 0,
        }
    }

    fn pill_target_snips(&self, block_size: u32) -> PillSnips {
        let mut context = BlockContext::new(0, block_size);
        let (width, height) = pill_size(block_size);
        PillSnips {
            shapes: self.pills.iter()
                .map(|(s, _)| (*s, context.next_unscaled(width, height)))
                .collect(),
            width: context.width(),
            height: context.height,
        }
    }

    fn dr_size(&self, dr_type: DrType) -> (u32, u32) {
        match dr_type {
            DrType::Normal => self.dr_normal_size,
            DrType::GameOver => self.dr_game_over_size,
            DrType::Victory => self.dr_victory_size,
            DrType::Wait => self.dr_wait_size,
        }
    }

    fn dr_points(&self, dr_type: DrType) -> &Vec<Point> {
        match dr_type {
            DrType::Normal => &self.dr_normal,
            DrType::GameOver => &self.dr_game_over,
            DrType::Victory => &self.dr_victory,
            DrType::Wait => &self.dr_wait
        }
    }

    fn dr_source_snips(&self, dr_type: DrType) -> Vec<Rect> {
        let (width, height) = self.dr_size(dr_type);
        self.dr_points(dr_type).iter().map(|p| Rect::new(p.x, p.y, width, height)).collect()
    }

    fn dr_sprites(&self, y: i32, dr_type: DrType) -> DrSnips {
        let mut context = BlockContext::new(y, self.source_block_size);

        let (width, height) = self.dr_size(dr_type);
        let snips = self.dr_points(dr_type).iter()
            .map(|_| context.next_unscaled(width, height))
            .collect();
        DrSnips {
            dr_type,
            snips,
            width: context.width(),
            height: context.height
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DrType {
    Normal,
    GameOver,
    Victory,
    Wait
}

struct DrSnips {
    dr_type: DrType,
    width: u32,
    height: u32,
    snips: Vec<Rect>
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

fn scale_pills<'a>(
    canvas: &mut WindowCanvas,
    data: &VitaminSpriteSheetData,
    src_texture: &Texture<'a>,
    target_texture: &mut Texture<'a>,
    target_snips: &PillSnips,
) -> Result<(), String> {
    canvas.with_texture_canvas(target_texture, |c| {
        let src_snips = data.pill_source_snips(data.source_block_size);
        for shape in PillShape::ALL {
            c.copy(&src_texture, src_snips.snip(shape), target_snips.snip(shape)).unwrap();
        }
    }).map_err(|e| e.to_string())
}

fn copy_dr<'a>(
    canvas: &mut WindowCanvas,
    data: &VitaminSpriteSheetData,
    src_texture: &Texture<'a>,
    target_texture: &mut Texture<'a>,
    target_snips: &DrSnips,
) -> Result<(), String> {
    canvas.with_texture_canvas(target_texture, |c| {
        let src_snips = data.dr_source_snips(target_snips.dr_type);
        for (src, target) in src_snips.into_iter().zip(target_snips.snips.iter().copied()) {
            c.copy(&src_texture, src, target).unwrap();
        }
    }).map_err(|e| e.to_string())
}

pub struct VitaminSpriteSheet<'a> {
    texture: Texture<'a>,
    alpha_textures: HashMap<u8, Texture<'a>>,
    ghost_alpha_mod: u8,
    yellow: BlockSnips,
    red: BlockSnips,
    blue: BlockSnips,
    pills: PillSnips,
    pill_texture: Texture<'a>,
    block_size: u32,
    dr: Texture<'a>,
    dr_normal: DrSnips,
    dr_game_over: DrSnips,
    dr_victory: DrSnips,
    dr_wait: DrSnips
}

impl<'a> VitaminSpriteSheet<'a> {
    pub fn new<B : Into<Option<u32>>>(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        data: VitaminSpriteSheetData,
        block_size: B
    ) -> Result<Self, String> {
        let block_size = block_size.into().unwrap_or(data.source_block_size);
        let sprite_src = texture_creator.load_texture_bytes(data.file)?;
        let yellow = data.target_snips(VirusColor::Yellow, 0, block_size);
        let red = data.target_snips(VirusColor::Red, yellow.height as i32, block_size);
        let blue = data.target_snips(VirusColor::Blue, (yellow.height + red.height) as i32, block_size);
        let width = yellow.width.max(red.width).max(blue.width);
        let height = yellow.height + red.height + blue.height;
        let mut texture = texture_creator
            .create_texture_target(None, width, height)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);

        scale_blocks(canvas, &data, &sprite_src, &mut texture, &yellow)?;
        scale_blocks(canvas, &data, &sprite_src, &mut texture, &red)?;
        scale_blocks(canvas, &data, &sprite_src, &mut texture, &blue)?;

        let mut alpha_textures = HashMap::new();
        for i in 0..0xff / ALPHA_STRIDE {
            let alpha_mod = i * ALPHA_STRIDE;
            let mut alpha_texture = texture_creator
                .create_texture_target(None, width, height)
                .map_err(|e| e.to_string())?;
            alpha_texture.set_blend_mode(BlendMode::Blend);
            alpha_texture.set_alpha_mod(alpha_mod);
            canvas
                .with_texture_canvas(&mut alpha_texture, |c| {
                    c.copy(&texture, None, None).unwrap();
                })
                .map_err(|e| e.to_string())?;
            alpha_textures.insert(alpha_mod, alpha_texture);
        }
        let ghost_alpha_mod = alpha_stride(data.ghost_alpha);

        let pills = data.pill_target_snips(block_size);
        let mut pill_texture = texture_creator
            .create_texture_target(None, pills.width, pills.height)
            .map_err(|e| e.to_string())?;
        pill_texture.set_blend_mode(BlendMode::Blend);
        scale_pills(canvas, &data, &sprite_src, &mut pill_texture, &pills)?;

        let dr_normal = data.dr_sprites(0, DrType::Normal);
        let dr_game_over = data.dr_sprites(dr_normal.height as i32, DrType::GameOver);
        let dr_victory = data.dr_sprites((dr_normal.height + dr_game_over.height) as i32, DrType::Victory);
        let dr_wait = data.dr_sprites((dr_normal.height + dr_game_over.height + dr_victory.height) as i32, DrType::Wait);
        let dr_width = dr_normal.width.max(dr_game_over.width).max(dr_victory.width).max(dr_wait.width);
        let dr_height = dr_normal.height + dr_game_over.height + dr_game_over.height + dr_wait.height;
        let mut dr = texture_creator
            .create_texture_target(None, dr_width, dr_height)
            .map_err(|e| e.to_string())?;
        dr.set_blend_mode(BlendMode::Blend);
        copy_dr(canvas, &data, &sprite_src, &mut dr, &dr_normal)?;
        copy_dr(canvas, &data, &sprite_src, &mut dr, &dr_game_over)?;
        copy_dr(canvas, &data, &sprite_src, &mut dr, &dr_victory)?;
        copy_dr(canvas, &data, &sprite_src, &mut dr, &dr_wait)?;

        Ok(Self { texture, alpha_textures, ghost_alpha_mod, yellow, red, blue, pills, pill_texture, block_size, dr, dr_normal, dr_game_over, dr_victory, dr_wait })
    }

    pub fn virus_frames(&self) -> usize {
        self.yellow.virus_frames.len()
            .min(self.red.virus_frames.len())
            .min(self.blue.virus_frames.len())
    }

    pub fn vitamin_pop_frames(&self) -> usize {
        self.yellow.vitamin_pop_frames.len()
            .min(self.red.vitamin_pop_frames.len())
            .min(self.blue.vitamin_pop_frames.len())
    }

    pub fn dr_frames(&self, dr_type: DrType) -> usize {
        self.dr_snips(dr_type).snips.len()
    }

    fn dr_snips(&self, dr_type: DrType) -> &DrSnips {
        match dr_type {
            DrType::Normal => &self.dr_normal,
            DrType::GameOver => &self.dr_game_over,
            DrType::Victory => &self.dr_victory,
            DrType::Wait => &self.dr_wait,
        }
    }

    pub fn draw_dr(&self, canvas: &mut WindowCanvas, dr_type: DrType, point: Point, frame: usize) -> Result<(), String> {
        let snip = self.dr_snips(dr_type).snips[frame];
        canvas.copy(&self.dr, snip, Rect::new(point.x, point.y, snip.width(), snip.height()))
    }

    /// TODO maybe move this back into the theme, it deals with animations and what not which is a theme concern
    pub fn draw_bottle(
        &self,
        canvas: &mut WindowCanvas,
        game: &Game,
        geometry: &BottleGeometry,
        animations: &PlayerAnimations
    ) -> Result<(), String> {
        let virus_frame = animations.virus().frame();

        if let Some(popping_in_viruses) = animations.next_level().state().and_then(|s| s.action().maybe_display_viruses().cloned()) {
            for virus in popping_in_viruses {
                let dest = geometry.raw_block(virus.position);
                canvas.copy(&self.texture, self.snips(virus.color).virus_frames[virus_frame], dest)?;
            }
            return Ok(());
        }

        let mut draw_vitamin = !animations.spawn().state().is_some();

        if let Some(hard_drop) = animations.hard_drop().state() {
            draw_vitamin = false;
            let vitamins = hard_drop.vitamins();
            for frame in hard_drop.frames() {
                for vitamin in vitamins {
                    let dest = geometry.raw_block(vitamin.position());
                    self.draw_vitamin(canvas, vitamin.color(), vitamin.rotation(), vitamin.ordinal(), dest, frame.offset_y, frame.alpha_mod)?;
                }
            }
        }

        let lock_animation = animations.lock()
            .state()
            .cloned()
            .unwrap_or_default();
        let lock_offset_y = lock_animation.offset_y();

        for j in (0..BOTTLE_HEIGHT).rev() {
            for (i, block) in game.row(j).iter().copied().enumerate() {
                let point = BottlePoint::new(i as i32, j as i32);
                let dest = geometry.raw_block(point);
                match block {
                    Block::Empty => {}
                    Block::Vitamin(color, rotation, ordinal) if draw_vitamin =>
                        self.draw_vitamin(canvas, color, rotation, ordinal, dest, 0.0, None)?,
                    Block::Stack(color, rotation, ordinal) => {
                        let offset_y = if lock_animation.animates(point) {
                            lock_offset_y
                        } else {
                            0.0
                        };
                        self.draw_vitamin(canvas, color, rotation, ordinal, dest, offset_y, None)?
                    }
                    Block::Garbage(color) => canvas.copy(&self.texture, self.snips(color).garbage, dest)?,
                    Block::Virus(color) => canvas.copy(&self.texture, self.snips(color).virus_frames[virus_frame], dest)?,
                    Block::Ghost(color, rotation, ordinal) if draw_vitamin =>
                        self.draw_vitamin(canvas, color, rotation, ordinal, dest, 0.0, self.ghost_alpha_mod)?,
                    _ => {}
                }
            }
        }

        if let Some(popped_vitamins) = animations.destroy().state() {
            let frame = popped_vitamins.frame();
            for block in popped_vitamins.blocks() {
                let dest = geometry.raw_block(block.position);
                canvas.copy(&self.texture, self.snips(block.color).vitamin_pop_frames[frame], dest)?;
            }
        }

        Ok(())
    }

    pub fn draw_pill<A: Into<Option<f64>>, S: Into<Option<f64>>>(
        &self,
        canvas: &mut WindowCanvas,
        shape: PillShape,
        point: Point,
        angle: A,
        scale: S
    ) -> Result<(), String> {
        let snip = self.pills.snip(shape);
        let mut dest = Rect::new(point.x, point.y, snip.width(), snip.height());
        if let Some(scale) = scale.into() {
            dest.scale_f64_mut(scale);
        }
        if let Some(angle) = angle.into() {
            canvas.copy_ex(&self.pill_texture, snip, dest, angle, None, false, false)
        } else {
            canvas.copy(&self.pill_texture, snip, dest)
        }
    }

    pub fn draw_vitamin<A : Into<Option<u8>>>(
        &self,
        canvas: &mut WindowCanvas,
        color: VirusColor,
        rotation: Rotation,
        ordinal: VitaminOrdinal,
        dest: Rect,
        offset_y: f64,
        alpha_mod: A
    ) -> Result<(), String> {
        let snip = self.snips(color).vitamin(rotation, ordinal);

        let texture = if let Some(alpha_mod) = alpha_mod.into() {
            self.alpha_texture(alpha_mod)
        } else {
            &self.texture
        };
        if offset_y > 0.0 || offset_y < 0.0 {
            let offset_dst = self.offset_by_block_ratio(dest, 0.0, offset_y);
            canvas.copy(texture, snip, offset_dst)
        } else {
            canvas.copy(texture, snip, dest)
        }
    }

    fn offset_by_block_ratio(
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

    fn snips(&self, color: VirusColor) -> &BlockSnips {
        match color {
            VirusColor::Yellow => &self.yellow,
            VirusColor::Blue => &self.blue,
            VirusColor::Red => &self.red
        }
    }

    fn alpha_texture(&self, alpha_mod: u8) -> &Texture {
        if let Some(exact) = self.alpha_textures.get(&alpha_mod) {
            exact
        } else {
            self.alpha_textures.get(&alpha_stride(alpha_mod)).unwrap()
        }
    }
}

trait Scalable {
    fn scale_f64(&self, factor: f64) -> Self;
    fn scale_f64_mut(&mut self, factor: f64);
}

impl Scalable for Rect {
    fn scale_f64(&self, factor: f64) -> Self {
        let mut result = self.clone();
        result.scale_f64_mut(factor);
        result
    }

    fn scale_f64_mut(&mut self, factor: f64) {
        self.resize(
            (self.width() as f64 * factor).round() as u32,
            (self.height() as f64 * factor).round() as u32
        )
    }
}