use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::rect::{Point, Rect};
use sdl2::video::WindowContext;
use sdl2::image::LoadTexture;

#[derive(Debug, Clone)]
enum FrameFormat {
    /// texture only contains a linear set of frames, nothing else
    /// i.e. the width of a frame is texture width / frames
    ///      & height is same as texture height
    ExclusiveLinear { count: u32 },

    /// animation is contained within a texture
    NonExclusiveLinear { start: Point, count: u32, width: u32, height: u32 }
}

#[derive(Debug, Clone)]
pub struct AnimationSpriteSheetData {
    file: &'static [u8],
    format: FrameFormat
}

impl AnimationSpriteSheetData {
    pub fn exclusive_linear(file: &'static [u8], frames: u32) -> Self {
        assert!(frames > 0);
        Self { file, format: FrameFormat::ExclusiveLinear { count: frames } }
    }

    pub fn non_exclusive_linear(file: &'static [u8], start: Point, frames: u32, frame_width: u32, frame_height: u32) -> Self {
        assert!(frames > 0 && frame_width > 0 && frame_height > 0);
        Self { file, format: FrameFormat::NonExclusiveLinear { start, count: frames, width: frame_width, height: frame_height } }
    }

    pub fn frame_count(&self) -> u32 {
        match self.format {
            FrameFormat::ExclusiveLinear { count, .. } => count,
            FrameFormat::NonExclusiveLinear { count, .. } => count
        }
    }

    pub fn sprite_sheet<'a>(&self, texture_creator: &'a TextureCreator<WindowContext>) -> Result<AnimationSpriteSheet<'a>, String> {
        let mut texture = texture_creator.load_texture_bytes(self.file)?;
        texture.set_blend_mode(BlendMode::Blend);

        let frames = match self.format {
            FrameFormat::ExclusiveLinear { count } => {
                let query = texture.query();
                let frame_width = query.width / count;
                (0..count).map(|i| Rect::new((i * frame_width) as i32, 0, frame_width, query.height)).collect()
            }
            FrameFormat::NonExclusiveLinear { start, count, width: frame_width, height: frame_height } => {
                (0..count).map(|i| Rect::new((i * frame_width) as i32 + start.x, start.y, frame_width, frame_height)).collect()
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
        // scale to a linear series of frames
        let mut texture = texture_creator
            .create_texture_target(None, width * self.frames.len() as u32, height)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);

        let mut frames = vec![];
        let mut frame = Rect::new(0, 0, width, height);
        canvas.with_texture_canvas(&mut texture, |c| {
            for src in self.frames.iter() {
                c.copy(&self.texture, *src, frame).unwrap();
                frames.push(frame.clone());
                frame.offset(width as i32, 0);
            }

        }).map_err(|e| e.to_string())?;

        Ok(AnimationSpriteSheet { texture, frames })
    }
}
