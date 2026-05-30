//! Widget de lista vertical interativa.

use crate::theme;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

/// Widget de lista vertical com detecção de toque e realce visual do item ativo.
pub struct ListWidget {
    items: Vec<String>,
    selected_idx: usize,
    bounds: Rectangle,
    item_height: u32,
}

impl ListWidget {
    /// Cria uma nova instância de ListWidget.
    pub fn new(items: Vec<String>, bounds: Rectangle, item_height: u32) -> Self {
        Self {
            items,
            selected_idx: 0,
            bounds,
            item_height,
        }
    }

    /// Desenha o widget de lista no buffer gráfico.
    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let border_style = PrimitiveStyleBuilder::new()
            .stroke_color(theme::TEXT_PRIMARY)
            .stroke_width(2)
            .build();
        self.bounds.into_styled(border_style).draw(target)?;

        let text_style_normal = MonoTextStyle::new(&FONT_8X13, theme::TEXT_PRIMARY);
        let text_style_selected = MonoTextStyle::new(&FONT_8X13, theme::BG);

        let selected_bg_style = PrimitiveStyleBuilder::new()
            .fill_color(theme::ACCENT)
            .build();

        for (i, item) in self.items.iter().enumerate() {
            let y_offset = self.bounds.top_left.y + (i as i32 * self.item_height as i32);
            if y_offset + self.item_height as i32 > self.bounds.bottom_right().unwrap().y {
                break;
            }

            let item_bounds = Rectangle::new(
                Point::new(self.bounds.top_left.x + 2, y_offset + 2),
                Size::new(self.bounds.size.width - 4, self.item_height - 4),
            );

            if i == self.selected_idx {
                item_bounds.into_styled(selected_bg_style).draw(target)?;
                Text::new(
                    item,
                    Point::new(item_bounds.top_left.x + 10, item_bounds.top_left.y + 15),
                    text_style_selected,
                )
                .draw(target)?;
            } else {
                Text::new(
                    item,
                    Point::new(item_bounds.top_left.x + 10, item_bounds.top_left.y + 15),
                    text_style_normal,
                )
                .draw(target)?;
            }
        }

        Ok(())
    }

    /// Processa o toque de tela e retorna o índice do item clicado, se houver.
    pub fn handle_touch(&mut self, x: u32, y: u32) -> Option<usize> {
        let pt = Point::new(x as i32, y as i32);
        if !self.bounds.contains(pt) {
            return None;
        }

        let relative_y = pt.y - self.bounds.top_left.y;
        let idx = (relative_y / self.item_height as i32) as usize;

        if idx < self.items.len() {
            self.selected_idx = idx;
            Some(idx)
        } else {
            None
        }
    }

    /// Retorna o índice do item selecionado no momento.
    pub fn selected_index(&self) -> usize {
        self.selected_idx
    }

    /// Altera manualmente o índice selecionado.
    pub fn set_selected(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.selected_idx = idx;
        }
    }
}
