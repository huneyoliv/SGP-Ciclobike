use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    text::Text,
};
use crate::theme;

pub struct ActionButtonWidget {
    bounds: Rectangle,
    label: &'static str,
    color: Rgb565,
}

impl ActionButtonWidget {
    pub fn new(bounds: Rectangle, label: &'static str, color: Rgb565) -> Self {
        Self {
            bounds,
            label,
            color,
        }
    }

    #[allow(clippy::manual_midpoint)]
    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let button_style = PrimitiveStyleBuilder::new()
            .fill_color(self.color)
            .stroke_color(theme::TEXT_PRIMARY)
            .stroke_width(2)
            .build();
        self.bounds.draw_styled(&button_style, target)?;

        let text_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_PRIMARY);
        let text_width = self.label.len() as i32 * 10;
        let text_height = 20;

        let center_x = self.bounds.top_left.x + (self.bounds.size.width as i32 - text_width) / 2;
        let center_y = self.bounds.top_left.y + (self.bounds.size.height as i32 + text_height) / 2 - 4;

        let _ = Text::new(
            self.label,
            Point::new(center_x, center_y),
            text_style,
        )
        .draw(target);

        Ok(())
    }

    pub fn contains(&self, p: Point) -> bool {
        self.bounds.contains(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;

    #[test]
    fn test_button_hit_test() {
        let button = ActionButtonWidget::new(
            Rectangle::new(Point::new(10, 10), Size::new(100, 50)),
            "OK",
            theme::ACCENT,
        );
        assert!(button.contains(Point::new(15, 15)));
        assert!(!button.contains(Point::new(5, 5)));
        
        let mut display = MockDisplay::<Rgb565>::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(button.draw(&mut display).is_ok());
    }
}
