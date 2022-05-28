use cairo_rs::{Context, FontSlant, FontWeight, Format, ImageSurface};

use crate::errors::AppError;

pub struct AvatarService {}

impl AvatarService {
    pub fn generate_avatar(text: &str) -> Result<ImageSurface, AppError> {
        let surface = ImageSurface::create(Format::ARgb32, 200, 200)
            .map_err(|error| AppError::avatat_generation_error(error))?;
        let cr =
            Context::new(&surface).map_err(|error| AppError::avatat_generation_error(error))?;

        cr.set_source_rgba(0.0, 0.79, 0.83, 0.8);
        cr.paint()
            .map_err(|error| AppError::avatat_generation_error(error))?;

        cr.select_font_face("Ubunut", FontSlant::Italic, FontWeight::Bold);
        cr.set_font_size(120.0);
        let extents = cr
            .text_extents(text)
            .map_err(|error| AppError::avatat_generation_error(error))?;

        let x = 100.0 - (extents.width() / 2.0 + extents.x_bearing());
        let y = 100.0 - (extents.height() / 2.0 + extents.y_bearing());
        cr.move_to(x, y);
        cr.set_source_rgb(100.0, 100.0, 100.0);
        cr.show_text("TA")
            .map_err(|error| AppError::avatat_generation_error(error))?;

        Ok(surface)
    }
}