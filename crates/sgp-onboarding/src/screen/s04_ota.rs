//! Tela 4: Verificação e Instalação de Atualização OTA.

use embedded_graphics::{
    prelude::*,
    text::Text,
    mono_font::ascii::FONT_8X13,
    text::MonoTextStyle,
    primitives::Rectangle,
};
use sgp_ui::framebuffer::FrameBuffer;
use sgp_ui::widgets::ProgressWidget;
use sgp_ui::theme;

/// Renderiza o status da verificação de atualizações OTA e progresso de download da nova firmware.
pub fn run(fb: &mut FrameBuffer, progress: u32, status: &str) {
    let _ = fb.draw_iter(std::iter::once(Pixel(Point::new(0, 0), theme::BG)));

    let title_style = MonoTextStyle::new(&FONT_8X13, theme::TEXT_PRIMARY);
    let _ = Text::new("ATUALIZACAO DO SISTEMA (OTA)", Point::new(40, 50), title_style).draw(fb);

    let status_style = MonoTextStyle::new(&FONT_8X13, theme::TEXT_PRIMARY);
    let _ = Text::new(status, Point::new(40, 100), status_style).draw(fb);

    let mut progress_bar = ProgressWidget::new(Rectangle::new(
        Point::new(40, 150),
        Size::new(460, 30),
    ));
    progress_bar.set_progress(progress);
    let _ = progress_bar.draw(fb);

    let percentage = format!("{progress}%");
    let _ = Text::new(&percentage, Point::new(240, 210), title_style).draw(fb);
}
