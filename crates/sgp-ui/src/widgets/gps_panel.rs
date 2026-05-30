use crate::theme;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    text::Text,
};

pub struct GpsPanelWidget {
    bounds: Rectangle,
    lat: f64,
    lon: f64,
    altitude_m: f32,
    satellites: u8,
}

impl GpsPanelWidget {
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            lat: 0.0,
            lon: 0.0,
            altitude_m: 0.0,
            satellites: 0,
        }
    }

    pub fn update(&mut self, lat: f64, lon: f64, altitude_m: f32, satellites: u8) {
        self.lat = lat;
        self.lon = lon;
        self.altitude_m = altitude_m;
        self.satellites = satellites;
    }

    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let card_style = PrimitiveStyleBuilder::new()
            .fill_color(theme::BG_CARD)
            .build();
        self.bounds.draw_styled(&card_style, target)?;

        let top_bar = Rectangle::new(self.bounds.top_left, Size::new(self.bounds.size.width, 2));
        let bar_style = PrimitiveStyleBuilder::new()
            .fill_color(theme::ACCENT_ALT)
            .build();
        top_bar.draw_styled(&bar_style, target)?;

        let label_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_SECONDARY);
        let _ = Text::new(
            "COORDENADAS GPS",
            Point::new(self.bounds.top_left.x + 8, self.bounds.top_left.y + 22),
            label_style,
        )
        .draw(target);

        if self.satellites == 0 {
            let warn_style = MonoTextStyle::new(&FONT_10X20, theme::WARNING);
            let _ = Text::new(
                "SEM SINAL GPS",
                Point::new(self.bounds.top_left.x + 8, self.bounds.top_left.y + 50),
                warn_style,
            )
            .draw(target);
        } else {
            let coord_str = format!("{:.4}, {:.4}", self.lat, self.lon);
            let alt_str = format!("{:.0}m (ALT)", self.altitude_m);
            let value_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_PRIMARY);

            let _ = Text::new(
                &coord_str,
                Point::new(self.bounds.top_left.x + 8, self.bounds.top_left.y + 44),
                value_style,
            )
            .draw(target);

            let _ = Text::new(
                &alt_str,
                Point::new(self.bounds.top_left.x + 8, self.bounds.top_left.y + 66),
                value_style,
            )
            .draw(target);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;

    #[test]
    fn test_gps_no_fix() {
        let mut widget = GpsPanelWidget::new(Rectangle::new(Point::new(0, 0), Size::new(200, 80)));
        widget.update(0.0, 0.0, 0.0, 0);
        let mut display = MockDisplay::<Rgb565>::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(widget.draw(&mut display).is_ok());
    }

    #[test]
    fn test_gps_with_fix() {
        let mut widget = GpsPanelWidget::new(Rectangle::new(Point::new(0, 0), Size::new(200, 80)));
        widget.update(-23.5505, -46.6333, 760.0, 8);
        let mut display = MockDisplay::<Rgb565>::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(widget.draw(&mut display).is_ok());
    }
}
