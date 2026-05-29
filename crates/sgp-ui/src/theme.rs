//! Definições de cores e estilo visual para a interface.

use embedded_graphics::pixelcolor::Rgb565;

/// Fundo preto profundo.
pub const BG: Rgb565 = Rgb565::new(0, 0, 0);

/// Texto principal (branco puro).
pub const TEXT_PRIMARY: Rgb565 = Rgb565::new(31, 63, 31);

/// Cor de destaque (laranja estilo Garmin).
pub const ACCENT: Rgb565 = Rgb565::new(31, 24, 0);

/// Cor indicativa de seleção (azul claro).
pub const SELECTED: Rgb565 = Rgb565::new(0, 40, 31);

/// Cor para indicação de erros e falhas.
pub const ERROR: Rgb565 = Rgb565::new(31, 0, 0);
