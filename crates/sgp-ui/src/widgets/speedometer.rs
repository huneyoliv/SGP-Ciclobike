use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    text::Text,
};
use crate::theme;

pub struct SpeedometerWidget {
    bounds: Rectangle,
    speed_kmh: f32,
}

impl SpeedometerWidget {
    pub fn new(bounds: Rectangle) -> Self {
        Self { bounds, speed_kmh: 0.0 }
    }

    pub fn set_speed(&mut self, kmh: f32) {
        self.speed_kmh = kmh.max(0.0);
    }

    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let card_style = PrimitiveStyleBuilder::new()
            .fill_color(theme::BG_CARD)
            .stroke_color(theme::ACCENT)
            .stroke_width(2)
            .build();
        self.bounds.draw_styled(&card_style, target)?;

        let label_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_SECONDARY);
        let _ = Text::new(
            "VELOCIDADE",
            Point::new(self.bounds.top_left.x + 16, self.bounds.top_left.y + 24),
            label_style,
        )
        .draw(target);

        let value_color = if self.speed_kmh > 0.0 { theme::ACCENT } else { theme::TEXT_SECONDARY };
        let value_style = MonoTextStyle::new(&FONT_10X20, value_color);
        let speed_str = format!("{:.1}", self.speed_kmh);
        let _ = Text::new(
            &speed_str,
            Point::new(self.bounds.top_left.x + 16, self.bounds.top_left.y + 140),
            value_style,
        )
        .draw(target);

        let unit_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_SECONDARY);
        let _ = Text::new(
            "KM/H",
            Point::new(self.bounds.top_left.x + 16, self.bounds.top_left.y + 170),
            unit_style,
        )
        .draw(target);

        let bar_y = self.bounds.top_left.y + self.bounds.size.height as i32 - 24;
        let bar_x = self.bounds.top_left.x + 16;
        let bar_max_w = self.bounds.size.width.saturating_sub(32);
        let fill_w = ((self.speed_kmh.min(60.0) / 60.0) * bar_max_w as f32) as u32;

        let track_style = PrimitiveStyleBuilder::new()
            .fill_color(theme::BG)
            .build();
        Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_max_w, 8))
            .draw_styled(&track_style, target)?;

        if fill_w > 0 {
            let fill_style = PrimitiveStyleBuilder::new()
                .fill_color(theme::ACCENT)
                .build();
            Rectangle::new(Point::new(bar_x, bar_y), Size::new(fill_w, 8))
                .draw_styled(&fill_style, target)?;
        }

        Ok(())
    }

    pub fn check_touch(&self, p: Point) -> bool {
        self.bounds.contains(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;

    #[test]
    fn test_speedometer_draw_zero() {
        let mut widget = SpeedometerWidget::new(
            Rectangle::new(Point::new(0, 0), Size::new(200, 100)),
        );
        widget.set_speed(0.0);
        let mut display = MockDisplay::<Rgb565>::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(widget.draw(&mut display).is_ok());
    }

    #[test]
    fn test_speedometer_draw_moving() {
        let mut widget = SpeedometerWidget::new(
            Rectangle::new(Point::new(0, 0), Size::new(200, 100)),
        );
        widget.set_speed(35.5);
        let mut display = MockDisplay::<Rgb565>::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(widget.draw(&mut display).is_ok());
    }
}
