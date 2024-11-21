use png::{BitDepth, ColorType, Info, Limits, Transformations};
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use std::time::Instant;
use sdl2::image::LoadTexture as SdlLoadTexture;

pub trait LoadTexture {
    fn load_texture_bytes(&self, buf: &[u8]) -> Result<Texture, String>;
}

impl<T> LoadTexture for TextureCreator<T> {
    fn load_texture_bytes(&self, png_buf: &[u8]) -> Result<Texture, String> {
        let now = Instant::now();

        let texture = surface_from_buffer(png_buf)?.as_texture(self).map_err(|e| e.to_string());
        // let texture =  SdlLoadTexture::load_texture_bytes(self, buf);

        println!("load texture: {:.2?}", now.elapsed());
        texture
    }
}

// pub fn surface_from_buffer2(png_buf: &[u8]) -> Result<Surface, String> {
//     let img = image::load_from_memory_with_format(png_buf, ImageFormat::Png)
//         .map_err(|e| e.to_string())?
//         .into_rgb8();
//
//     let width = img.width();
//     let height = img.height();
//
//     Surface::from_data(img.into_raw().as_mut_slice(), width, height, width * 4, PixelFormatEnum::RGBA8888)
// }

pub fn surface_from_buffer(png_buf: &[u8]) -> Result<Surface, String> {
    let mut decoder = png::Decoder::new(png_buf);
    let mut reader = decoder.read_info()
        .map_err(|e| e.to_string())?;

    let info = reader.info();
    let (color_type, bit_depth) = reader.output_color_type();
    let pixel_format = match color_type.samples() {
        3 => PixelFormatEnum::RGB24,
        4 => PixelFormatEnum::RGBA32,
        1 => match bit_depth as usize * color_type.samples() {
            1 => PixelFormatEnum::Index1MSB,
            4 => PixelFormatEnum::Index4MSB,
            8 => PixelFormatEnum::Index8,
            12 => PixelFormatEnum::RGB444,
            15 => PixelFormatEnum::RGB555,
            16 => PixelFormatEnum::RGB565,
            _ => PixelFormatEnum::Unknown
        },
        _ => PixelFormatEnum::Unknown
    };

    let mut surface = Surface::new(info.width, info.height, pixel_format)?;
    if info.palette.is_some() {
        let palette = create_rgba_palette(info);
        let colors = palette.into_iter()
            .map(|[r, g, b, a]| Color::RGBA(r, g, b, a))
            .collect::<Vec<Color>>();
        let sdl_palette = Palette::with_colors(&colors)?;
        surface.set_palette(&sdl_palette)?;
    }

    reader.next_frame(surface.without_lock_mut().unwrap()).map_err(|e| e.to_string())?;

    Ok(surface)
}

fn create_rgba_palette(info: &Info) -> [[u8; 4]; 256] {
    let palette = info.palette.as_deref().expect("Caller should verify");
    let trns = info.trns.as_deref().unwrap_or(&[]);

    // > The tRNS chunk shall not contain more alpha values than there are palette
    // entries, but a tRNS chunk may contain fewer values than there are palette
    // entries. In this case, the alpha value for all remaining palette entries is
    // assumed to be 255.
    //
    // It seems, accepted reading is to fully *ignore* an invalid tRNS as if it were
    // completely empty / all pixels are non-transparent.
    let trns = if trns.len() <= palette.len() / 3 {
        trns
    } else {
        &[]
    };

    // Default to black, opaque entries.
    let mut rgba_palette = [[0, 0, 0, 0xFF]; 256];

    // Copy `palette` (RGB) entries into `rgba_palette`.  This may clobber alpha
    // values in `rgba_palette` - we need to fix this later.
    {
        let mut palette_iter = palette;
        let mut rgba_iter = &mut rgba_palette[..];
        while palette_iter.len() >= 4 {
            // Copying 4 bytes at a time is more efficient than copying 3.
            // OTOH, this clobbers the alpha value in `rgba_iter[0][3]` - we
            // need to fix this later.
            rgba_iter[0].copy_from_slice(&palette_iter[0..4]);

            palette_iter = &palette_iter[3..];
            rgba_iter = &mut rgba_iter[1..];
        }
        if !palette_iter.is_empty() {
            rgba_iter[0][0..3].copy_from_slice(&palette_iter[0..3]);
        }
    }

    // Copy `trns` (alpha) entries into `rgba_palette`.  `trns.len()` may be
    // smaller than `palette.len()` and therefore this is not sufficient to fix
    // all the clobbered alpha values.
    for (alpha, rgba) in trns.iter().copied().zip(rgba_palette.iter_mut()) {
        rgba[3] = alpha;
    }

    // Unclobber the remaining alpha values.
    for rgba in rgba_palette[trns.len()..(palette.len() / 3)].iter_mut() {
        rgba[3] = 0xFF;
    }

    rgba_palette
}
