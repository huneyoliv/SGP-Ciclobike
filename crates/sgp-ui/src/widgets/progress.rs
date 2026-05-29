//! Widget de barra de progresso horizontal.

use embedded_graphics::{
    prelude::*,
    primitives::{Rectangle, PrimitiveStyleBuilder},
    pixelcolor::Rgb565,
};
use crate::theme;

/// Widget de barra de progresso horizontal para telas de download ou loading.
pub struct ProgressWidget {
    bounds: Rectangle,
    progress: u32,
}

impl ProgressWidget {
    /// Cria uma nova instância de ProgressWidget.
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            progress: 0,
        }
    }

    /// Altera o progresso atual (deve estar contido entre 0 e 100).
    pub fn set_progress(&mut self, progress: u32) {
        self.progress = progress.min(100);
    }

    /// Desenha o widget no target gráfico fornecido.
    pub fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        let border_style = PrimitiveStyleBuilder::new()
            .stroke_color(theme::TEXT_PRIMARY)
            .stroke_width(2)
            .build();
        self.bounds.into_styled(border_style).draw(target)?;

        if self.progress > 0 {
            let fill_width = ((self.bounds.size.width - 6) * self.progress) / 100;
            if fill_width > 0 {
                let fill_bounds = Rectangle::new(
                    Point::new(self.bounds.top_left.x + 3, self.bounds.top_left.y + 3),
                    Size::new(fill_width, self.bounds.size.height - 6),
                );

                let fill_style = PrimitiveStyleBuilder::new()
                    .fill_color(theme::ACCENT)
                    .build();

                fill_bounds.into_styled(fill_style).draw(target)?;
            }
        }

        Ok(())
    }
}
