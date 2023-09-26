use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::rect::{Point, Rect};
use sdl2::video::WindowContext;
use sdl2::image::LoadTexture;
use crate::theme::block_mask::BlockMask;
use crate::theme::helper::{TextureFactory, TextureQuery};

#[derive(Debug, Clone)]
enum FrameFormat {
    /// texture only contains a linear set of square frames, nothing else
    /// i.e. the width of a frame == height of frame == height of texture
    ExclusiveSquareLinear,

    /// texture only contains a linear set of frames, nothing else
    /// i.e. the width of a frame is texture width / frames
    ///      & height is same as texture height
    ExclusiveLinear { count: u32 },

    /// animation is contained within a texture
    NonExclusiveLinear { start: Point, count: u32, width: u32, height: u32 },

    /// texture only contains a linear set of square frames, nothing else
    /// static means only use the first frame
    StaticExclusiveSquare { frame: u32 },

    /// A table of sprites
    ExclusiveTable { rows: u32, cols: u32, count: u32 },
}

#[derive(Debug, Clone)]
pub struct AnimationSpriteSheetData {
    file: &'static [u8],
    format: FrameFormat
}

impl AnimationSpriteSheetData {
    pub fn exclusive_square_linear(file: &'static [u8]) -> Self {
        Self { file, format: FrameFormat::ExclusiveSquareLinear }
    }

    pub fn exclusive_linear(file: &'static [u8], frames: u32) -> Self {
        assert!(frames > 0);
        Self { file, format: FrameFormat::ExclusiveLinear { count: frames } }
    }

    pub fn exclusive_table(file: &'static [u8], rows: u32, cols: u32, frames: u32) -> Self {
        assert!(rows > 0 && cols > 0);
        Self { file, format: FrameFormat::ExclusiveTable { rows, cols, count: frames } }
    }

    pub fn non_exclusive_linear(file: &'static [u8], start: Point, frames: u32, frame_width: u32, frame_height: u32) -> Self {
        assert!(frames > 0 && frame_width > 0 && frame_height > 0);
        Self { file, format: FrameFormat::NonExclusiveLinear { start, count: frames, width: frame_width, height: frame_height } }
    }

    pub fn static_first_square_frame(file: &'static [u8]) -> Self {
        Self { file, format: FrameFormat::StaticExclusiveSquare { frame: 0 } }
    }

    pub fn frame_count(&self) -> Option<u32> {
        match self.format {
            FrameFormat::ExclusiveLinear { count, .. } => Some(count),
            FrameFormat::NonExclusiveLinear { count, .. } => Some(count),
            FrameFormat::ExclusiveTable { count, .. } => Some(count),
            _ => None
        }
    }

    pub fn sprite_sheet<'a>(&self, texture_creator: &'a TextureCreator<WindowContext>) -> Result<AnimationSpriteSheet<'a>, String> {
        let mut texture = texture_creator.load_texture_bytes_blended(self.file)?;
        let (texture_width, texture_height) = texture.size();

        let frames = match self.format {
            FrameFormat::ExclusiveLinear { count } => {
                let frame_width = texture_width / count;
                (0..count).map(|i| Rect::new((i * frame_width) as i32, 0, frame_width, texture_height)).collect()
            }
            FrameFormat::NonExclusiveLinear { start, count, width: frame_width, height: frame_height } => {
                (0..count).map(|i| Rect::new((i * frame_width) as i32 + start.x, start.y, frame_width, frame_height)).collect()
            }
            FrameFormat::ExclusiveSquareLinear => {
                let frame_size = texture_height;
                let count = texture_width / frame_size;
                (0..count).map(|i| Rect::new((i * frame_size) as i32, 0, frame_size, frame_size)).collect()
            }
            FrameFormat::StaticExclusiveSquare { frame } => {
                let frame_size = texture_height;
                vec![Rect::new((frame * frame_size) as i32, 0, frame_size, frame_size)]
            }
            FrameFormat::ExclusiveTable { rows, cols, count } => {
                let frame_width = texture_width / cols;
                let frame_height = texture_height / rows;
                let mut result = vec![];

                for j in 0..rows as i32 {
                    for i in 0..cols as i32 {
                        result.push(
                            Rect::new(
                                i * frame_width as i32, j * frame_height as i32,
                                frame_width, frame_height
                            )
                        );
                        if result.len() as u32 == count {
                            break;
                        }
                    }
                    if result.len() as u32 == count {
                        break;
                    }
                }

                result
            }
        };

        Ok(AnimationSpriteSheet { texture, frames })
    }
}

pub struct AnimationSpriteSheet<'a> {
    texture: Texture<'a>,
    frames: Vec<Rect>
}

impl<'a> AnimationSpriteSheet<'a> {
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn frame_size(&self) -> (u32, u32) {
        let first_frame = self.frames.first().expect("empty animation");
        (first_frame.width(), first_frame.height())
    }

    pub fn draw_frame(&self, canvas: &mut WindowCanvas, dest: Point, frame: usize) -> Result<(), String> {
        let snip = self.frames[frame];
        canvas.copy(&self.texture, snip, Rect::new(dest.x, dest.y, snip.width(), snip.height()))
    }

    pub fn draw_frame_scaled(&self, canvas: &mut WindowCanvas, dest: Rect, frame: usize) -> Result<(), String> {
        let snip = self.frames[frame];
        canvas.copy(&self.texture, snip, dest)
    }

    pub fn scale<'b>(&self,
                     canvas: &mut WindowCanvas,
                     texture_creator: &'b TextureCreator<WindowContext>,
                     width: u32,
                     height: u32
    ) -> Result<AnimationSpriteSheet<'b>, String> {
        // sdl has a texture size limit so scale to a square of frames
        let frame_len_sqrt = (self.frames.len() as f64).sqrt();
        let cols = frame_len_sqrt.floor() as u32;
        let rows = frame_len_sqrt.ceil() as u32;

        let scaled_frames = self.frames.iter().enumerate()
            .map(|(i, src)| {
                let col = i as u32 % cols;
                let row = i as u32 / cols;
                Rect::new((col * width) as i32, (row * height) as i32, width, height)
            })
            .collect::<Vec<Rect>>();

        let mut texture = texture_creator.create_texture_target_blended(width * (cols + 1), height * (rows + 1))?;
        canvas.with_texture_canvas(&mut texture, |c| {
            for (src, dest) in self.frames.iter().zip(scaled_frames.iter()) {
                c.copy(&self.texture, *src, *dest).unwrap();
            }
        }).map_err(|e| e.to_string())?;

        Ok(AnimationSpriteSheet { texture, frames: scaled_frames })
    }

    pub fn scale_f64<'b>(&self,
                         canvas: &mut WindowCanvas,
                         texture_creator: &'b TextureCreator<WindowContext>,
                         scale: f64
    ) -> Result<AnimationSpriteSheet<'b>, String> {
        let (frame_width, frame_height) = self.frame_size();
        let width = (frame_width as f64 * scale).round() as u32;
        let height = (frame_height as f64 * scale).round() as u32;

        self.scale(canvas, texture_creator, width, height)
    }

    pub fn block_mask(&mut self, canvas: &mut WindowCanvas, frame: usize) -> Result<BlockMask, String> {
        BlockMask::from_texture(canvas, &mut self.texture, self.frames[frame])
    }
}
