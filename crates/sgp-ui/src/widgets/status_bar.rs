use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    text::Text,
};
use crate::theme;

pub struct StatusBarWidget {
    width: u32,
    satellites: u8,
    session_seconds: u32,
    recording: bool,
}

impl StatusBarWidget {
    pub fn new(width: u32) -> Self {
        Self {
            width,
            satellites: 0,
            session_seconds: 0,
            recording: false,
        }
    }

    pub fn update(&mut self, satellites: u8, session_seconds: u32, recording: bool) {
        self.satellites = satellites;
        self.session_seconds = session_seconds;
        self.recording = recording;
    }

    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let bg_style = PrimitiveStyleBuilder::new()
            .fill_color(theme::BG_CARD)
            .build();
        Rectangle::new(Point::new(0, 0), Size::new(self.width, 32))
            .draw_styled(&bg_style, target)?;

        let gps_color = match self.satellites {
            0 => theme::ERROR,
            1..=3 => theme::WARNING,
            _ => theme::SUCCESS,
        };
        let dot_style = PrimitiveStyleBuilder::new()
            .fill_color(gps_color)
            .build();
        Circle::new(Point::new(8, 12), 8).draw_styled(&dot_style, target)?;

        let gps_str = format!("GPS {}sat", self.satellites);
        let text_style = MonoTextStyle::new(&FONT_10X20, theme::TEXT_PRIMARY);
        let _ = Text::new(&gps_str, Point::new(22, 22), text_style).draw(target);

        let h = self.session_seconds / 3600;
        let m = (self.session_seconds % 3600) / 60;
        let s = self.session_seconds % 60;
        let timer_str = format!("{h:02}:{m:02}:{s:02}");
        let _ = Text::new(&timer_str, Point::new(160, 22), text_style).draw(target);

        let _ = Text::new("CICLOBIKE", Point::new(300, 22), text_style).draw(target);

        if self.recording {
            let rec_dot_style = PrimitiveStyleBuilder::new()
                .fill_color(theme::ERROR)
                .build();
            Circle::new(Point::new(self.width as i32 - 24, 12), 8)
                .draw_styled(&rec_dot_style, target)?;
            let _ = Text::new(
                "REC",
                Point::new(self.width as i32 - 56, 22),
                MonoTextStyle::new(&FONT_10X20, theme::ERROR),
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
    fn test_status_bar_no_gps() {
        let mut bar = StatusBarWidget::new(540);
        bar.update(0, 0, false);
        let mut display = MockDisplay::<Rgb565>::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(bar.draw(&mut display).is_ok());
     }

    #[test]
    fn test_status_bar_ok() {
        let mut bar = StatusBarWidget::new(540);
        bar.update(9, 735, true);
        let mut display = MockDisplay::<Rgb565>::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(bar.draw(&mut display).is_ok());
    }
}
