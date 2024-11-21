use sdl2::surface::Surface;
use crate::theme::image::surface_from_buffer;

const ICON_FILE: &[u8] = include_bytes!("../icon.png");


pub fn app_icon() -> Result<Surface<'static>, String> {
    surface_from_buffer(ICON_FILE)
}
