use crate::theme::helper::decode_png;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;

const ICON_FILE: &[u8] = include_bytes!("../icon.png");

pub fn app_icon() -> Result<Surface<'static>, String> {
    let image = decode_png(ICON_FILE)?;
    let (width, height) = image.dimensions();
    let mut surface = Surface::new(width, height, PixelFormatEnum::RGBA32)?;
    surface.with_lock_mut(|pixels| pixels.copy_from_slice(image.as_raw()));
    Ok(surface)
}
