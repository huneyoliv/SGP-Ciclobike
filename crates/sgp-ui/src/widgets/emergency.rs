use crate::theme;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable, Triangle},
    text::Text,
};

pub struct EmergencyAlertWidget {
    bounds: Rectangle,
    emergency_number: String,
}

impl EmergencyAlertWidget {
    pub fn new(bounds: Rectangle, emergency_number: &str) -> Self {
        Self {
            bounds,
            emergency_number: emergency_number.to_string(),
        }
    }

    pub fn draw<D>(
        &self,
        target: &mut D,
        seconds_remaining: u8,
        flash_state: bool,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let bg_color = if flash_state {
            Rgb565::new(28, 0, 0)
        } else {
            Rgb565::new(12, 0, 0)
        };

        let bg_style = PrimitiveStyleBuilder::new().fill_color(bg_color).build();
        self.bounds.draw_styled(&bg_style, target)?;

        let triangle_top = Point::new(270, 80);
        let triangle_left = Point::new(210, 180);
        let triangle_right = Point::new(330, 180);

        let tri_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::new(31, 56, 0))
            .build();
        Triangle::new(triangle_top, triangle_left, triangle_right)
            .draw_styled(&tri_style, target)?;

        let exclamation_style = MonoTextStyle::new(&FONT_10X20, Rgb565::new(0, 0, 0));
        let _ = Text::new("!", Point::new(266, 145), exclamation_style).draw(target);

        let text_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_PRIMARY);
        let _ = Text::new("QUEDA DETECTADA!", Point::new(180, 230), text_style).draw(target);

        let countdown_str = format!("Chamando em {seconds_remaining}s...");
        let _ = Text::new(&countdown_str, Point::new(185, 270), text_style).draw(target);

        let contact_str = format!("Servicos locais: {}", self.emergency_number);
        let _ = Text::new(&contact_str, Point::new(170, 310), text_style).draw(target);

        let button_rect = Rectangle::new(Point::new(120, 370), Size::new(300, 60));
        let button_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::new(0, 48, 4))
            .build();
        button_rect.draw_styled(&button_style, target)?;

        let button_text_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_PRIMARY);
        let _ = Text::new(
            "ESTOU BEM (CANCELAR)",
            Point::new(165, 405),
            button_text_style,
        )
        .draw(target);

        Ok(())
    }

    pub fn check_touch(&self, touch_point: Point) -> bool {
        touch_point.x >= 120 && touch_point.x <= 420 && touch_point.y >= 370 && touch_point.y <= 430
    }
}
