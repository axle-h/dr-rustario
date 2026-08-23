use crate::render::helper::decode_png;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;

/// Decode an embedded PNG into an SDL surface suitable for `Window::set_icon`.
pub fn app_icon(png: &[u8]) -> Result<Surface<'static>, String> {
    let image = decode_png(png)?;
    let (width, height) = image.dimensions();
    let mut surface = Surface::new(width, height, PixelFormatEnum::RGBA32)?;
    surface.with_lock_mut(|pixels| pixels.copy_from_slice(image.as_raw()));
    Ok(surface)
}
