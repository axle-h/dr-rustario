use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};

const BASE_TRANSPARENCY: u8 = 0;

#[derive(Debug, Clone)]
pub struct BlockMask {
    width: u32,
    height: u32,
    mask: Vec<bool>,
}

impl BlockMask {
    pub fn from_texture(
        canvas: &mut WindowCanvas,
        texture: &mut Texture,
        rect: Rect,
    ) -> Result<Self, String> {
        let mut pixels = vec![];
        canvas
            .with_texture_canvas(texture, |c| {
                pixels = c.read_pixels(rect, PixelFormatEnum::ARGB8888).unwrap()
            })
            .map_err(|e| e.to_string())?;

        let mask = pixels
            .as_slice()
            .chunks(4)
            .map(|chunk| chunk[0] > BASE_TRANSPARENCY)
            .collect();

        Ok(Self {
            width: rect.width(),
            height: rect.height(),
            mask,
        })
    }

    pub fn lattice(&self, offset: Point, spacing: u32) -> Vec<Point> {
        let mut result = vec![];
        for j in 0..self.height / spacing {
            let y = (j * spacing) as i32;
            for i in 0..self.width / spacing {
                let x = (i * spacing) as i32;
                if self.mask[y as usize * self.height as usize + x as usize] {
                    let point = Point::new(offset.x() + x, offset.y() + y);
                    result.push(point);
                }
            }
        }
        result
    }
}
