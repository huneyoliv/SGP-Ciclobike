//! Tela 1: Seleção de Idioma do Wizard.

use embedded_graphics::{
    prelude::*,
    text::Text,
    mono_font::ascii::FONT_8X13,
    text::MonoTextStyle,
    primitives::Rectangle,
};
use sgp_core::LanguageCode;
use sgp_ui::framebuffer::FrameBuffer;
use sgp_ui::widgets::ListWidget;
use sgp_ui::theme;

/// Renderiza a tela de seleção de idioma e processa eventos de toque.
pub fn run(fb: &mut FrameBuffer, touch: Option<(u32, u32)>) -> Option<LanguageCode> {
    let _ = fb.draw_iter(std::iter::once(Pixel(Point::new(0, 0), theme::BG)));

    let title_style = MonoTextStyle::new(&FONT_8X13, theme::TEXT_PRIMARY);
    let _ = Text::new("SELECIONE O IDIOMA", Point::new(40, 50), title_style).draw(fb);

    let items = vec![
        "1. Português (pt-BR)".to_string(),
        "2. English (en-US)".to_string(),
        "3. Español (es-ES)".to_string(),
    ];

    let mut list = ListWidget::new(
        items,
        Rectangle::new(Point::new(20, 100), Size::new(500, 200)),
        50,
    );

    let mut result = None;
    if let Some((x, y)) = touch {
        if let Some(idx) = list.handle_touch(x, y) {
            let code = match idx {
                0 => "pt-BR",
                1 => "en-US",
                _ => "es-ES",
            };
            result = Some(LanguageCode::new(code).unwrap());
        }
    }

    let _ = list.draw(fb);
    result
}
