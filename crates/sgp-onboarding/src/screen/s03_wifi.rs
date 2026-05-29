//! Tela 3: Configuração e Conexão de Rede Wi-Fi.

use embedded_graphics::{
    prelude::*,
    text::Text,
    mono_font::ascii::FONT_8X13,
    text::MonoTextStyle,
    primitives::Rectangle,
};
use sgp_ui::framebuffer::FrameBuffer;
use sgp_ui::widgets::KeyboardWidget;
use sgp_ui::theme;

/// Renderiza o teclado virtual e caixa de texto de senha do Wi-Fi, processando a entrada do usuário.
pub fn run(
    fb: &mut FrameBuffer,
    keyboard: &mut KeyboardWidget,
    touch: Option<(u32, u32)>,
) -> Option<String> {
    let _ = fb.draw_iter(std::iter::once(Pixel(Point::new(0, 0), theme::BG)));

    let title_style = MonoTextStyle::new(&FONT_8X13, theme::TEXT_PRIMARY);
    let _ = Text::new("DIGITE A SENHA DO WI-FI", Point::new(40, 50), title_style).draw(fb);
    let _ = Text::new("Rede: Ciclobike_WiFi", Point::new(40, 80), title_style).draw(fb);

    let text_box_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .stroke_color(theme::TEXT_PRIMARY)
        .stroke_width(2)
        .build();

    let text_box = Rectangle::new(Point::new(40, 110), Size::new(460, 40));
    let _ = text_box.draw_styled(&text_box_style, fb);

    let password_mask = "*".repeat(keyboard.text().len());
    let _ = Text::new(
        &password_mask,
        Point::new(50, 135),
        MonoTextStyle::new(&FONT_8X13, theme::TEXT_PRIMARY),
    )
    .draw(fb);

    let mut confirmed = false;
    if let Some((x, y)) = touch {
        if keyboard.handle_touch(x, y) {
            confirmed = true;
        }
    }

    let _ = keyboard.draw(fb);

    if confirmed {
        Some("Ciclobike_WiFi".to_string())
    } else {
        None
    }
}
