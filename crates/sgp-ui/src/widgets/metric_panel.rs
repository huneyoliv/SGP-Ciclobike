use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    text::Text,
};
use crate::theme;

pub struct MetricPanelWidget {
    bounds: Rectangle,
    label: &'static str,
    unit: &'static str,
    value: f32,
}

impl MetricPanelWidget {
    pub fn new(bounds: Rectangle, label: &'static str, unit: &'static str) -> Self {
        Self { bounds, label, unit, value: 0.0 }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let card_style = PrimitiveStyleBuilder::new()
            .fill_color(theme::BG_CARD)
            .build();
        self.bounds.draw_styled(&card_style, target)?;

        let top_bar = Rectangle::new(
            self.bounds.top_left,
            Size::new(self.bounds.size.width, 2),
        );
        let bar_style = PrimitiveStyleBuilder::new()
            .fill_color(theme::ACCENT_ALT)
            .build();
        top_bar.draw_styled(&bar_style, target)?;

        let label_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_SECONDARY);
        let _ = Text::new(
            self.label,
            Point::new(self.bounds.top_left.x + 8, self.bounds.top_left.y + 22),
            label_style,
        )
        .draw(target);

        let value_str = format!("{:.1} {}", self.value, self.unit);
        let value_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_PRIMARY);
        let _ = Text::new(
            &value_str,
            Point::new(self.bounds.top_left.x + 8, self.bounds.top_left.y + 50),
            value_style,
        )
        .draw(target);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;

    #[test]
    fn test_metric_panel_render() {
        let mut panel = MetricPanelWidget::new(
            Rectangle::new(Point::new(0, 0), Size::new(160, 80)),
            "CADÊNCIA",
            "RPM",
        );
        panel.set_value(82.0);
        let mut display = MockDisplay::<Rgb565>::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(panel.draw(&mut display).is_ok());
    }
}
