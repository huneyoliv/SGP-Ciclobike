//! Widget de teclado virtual QWERTY.

use crate::theme;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

const ROWS: usize = 5;
const COLS: usize = 10;

/// Teclado alfanumérico virtual para digitação de senhas e dados na tela de toque.
pub struct KeyboardWidget {
    keys: [[&'static str; COLS]; ROWS],
    bounds: Rectangle,
    key_width: u32,
    key_height: u32,
    buffer: String,
}

impl KeyboardWidget {
    /// Cria uma nova instância de KeyboardWidget.
    pub fn new(bounds: Rectangle) -> Self {
        let keys = [
            ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
            ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
            ["a", "s", "d", "f", "g", "h", "j", "k", "l", "-"],
            ["z", "x", "c", "v", "b", "n", "m", "_", ".", "@"],
            [
                "Spc", "Spc", "Spc", "Del", "Del", "Del", "Ent", "Ent", "Ent", "Ent",
            ],
        ];
        let key_width = bounds.size.width / COLS as u32;
        let key_height = bounds.size.height / ROWS as u32;

        Self {
            keys,
            bounds,
            key_width,
            key_height,
            buffer: String::new(),
        }
    }

    /// Desenha o teclado virtual e a caixa de texto do buffer de entrada.
    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let border_style = PrimitiveStyleBuilder::new()
            .stroke_color(theme::TEXT_PRIMARY)
            .stroke_width(1)
            .build();

        let text_style = MonoTextStyle::new(&FONT_8X13, theme::TEXT_PRIMARY);

        for row in 0..ROWS {
            for col in 0..COLS {
                let key_label = self.keys[row][col];

                if row == 4 {
                    if col > 0 && self.keys[row][col] == self.keys[row][col - 1] {
                        continue;
                    }
                }

                let (width_multiplier, draw_label) = match (row, col) {
                    (4, 0) => (3, "Espaço"),
                    (4, 3) => (3, "Apagar"),
                    (4, 6) => (4, "Confirmar"),
                    _ => (1, key_label),
                };

                let x = self.bounds.top_left.x + (col as i32 * self.key_width as i32);
                let y = self.bounds.top_left.y + (row as i32 * self.key_height as i32);

                let key_bounds = Rectangle::new(
                    Point::new(x, y),
                    Size::new(self.key_width * width_multiplier, self.key_height),
                );

                key_bounds.into_styled(border_style).draw(target)?;

                Text::new(
                    draw_label,
                    Point::new(x + 10, y + (self.key_height as i32 / 2) + 5),
                    text_style,
                )
                .draw(target)?;
            }
        }

        Ok(())
    }

    /// Processa o toque nas teclas virtuais. Retorna se a tecla pressionada foi o "Enter" (confirmação).
    pub fn handle_touch(&mut self, x: u32, y: u32) -> bool {
        let pt = Point::new(x as i32, y as i32);
        if !self.bounds.contains(pt) {
            return false;
        }

        let rel_x = pt.x - self.bounds.top_left.x;
        let rel_y = pt.y - self.bounds.top_left.y;

        let col = (rel_x / self.key_width as i32) as usize;
        let row = (rel_y / self.key_height as i32) as usize;

        if row < ROWS && col < COLS {
            let key = self.keys[row][col];
            match key {
                "Spc" => {
                    self.buffer.push(' ');
                }
                "Del" => {
                    self.buffer.pop();
                }
                "Ent" => {
                    return true;
                }
                normal => {
                    self.buffer.push_str(normal);
                }
            }
        }
        false
    }

    /// Retorna o conteúdo digitado até o momento.
    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// Limpa o buffer de texto do teclado.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
