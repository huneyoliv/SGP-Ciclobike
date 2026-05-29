//! Tela 2: Seleção de País e Configuração Regional do Wizard.

use embedded_graphics::{
    prelude::*,
    text::Text,
    mono_font::ascii::FONT_8X13,
    text::MonoTextStyle,
    primitives::Rectangle,
};
use sgp_core::CountryCode;
use sgp_ui::framebuffer::FrameBuffer;
use sgp_ui::widgets::ListWidget;
use sgp_ui::theme;

/// Renderiza a tela de seleção de país e processa eventos de toque.
pub fn run(fb: &mut FrameBuffer, touch: Option<(u32, u32)>) -> Option<CountryCode> {
    let _ = fb.draw_iter(std::iter::once(Pixel(Point::new(0, 0), theme::BG)));

    let title_style = MonoTextStyle::new(&FONT_8X13, theme::TEXT_PRIMARY);
    let _ = Text::new("SELECIONE O PAIS", Point::new(40, 50), title_style).draw(fb);

    let items = vec![
        "Brasil (SAMU 192)".to_string(),
        "United States (911)".to_string(),
        "España (112)".to_string(),
    ];

    let mut list = ListWidget::new(
        items,
        Rectangle::new(Point::new(20, 100), Size::new(500, 200)),
        50,
    );

    let mut result = None;
    if let Some((x, y)) = touch {
        if let Some(idx) = list.handle_touch(x, y) {
            result = match idx {
                0 => Some(CountryCode {
                    iso2: "BR".to_string(),
                    emergency_number: "192".to_string(),
                    map_url: "https://example.com/maps/brazil.map".to_string(),
                }),
                1 => Some(CountryCode {
                    iso2: "US".to_string(),
                    emergency_number: "911".to_string(),
                    map_url: "https://example.com/maps/usa.map".to_string(),
                }),
                _ => Some(CountryCode {
                    iso2: "ES".to_string(),
                    emergency_number: "112".to_string(),
                    map_url: "https://example.com/maps/spain.map".to_string(),
                }),
            };
        }
    }

    let _ = list.draw(fb);
    result
}
