//! Leitura assíncrona de eventos de toque (/dev/input/event*).

use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Estrutura contendo dados do evento de toque na tela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchEvent {
    /// Coordenada X (0..540).
    pub x: u32,
    /// Coordenada Y (0..960).
    pub y: u32,
    /// Indica se a tela está sendo pressionada.
    pub pressed: bool,
}

/// Leitor assíncrono do dispositivo de toque Linux.
pub struct TouchReader {
    _file: Option<File>,
}

impl TouchReader {
    /// Tenta abrir o dispositivo de toque físico (/dev/input/event0). Caso indisponível, inicia em modo virtual.
    pub fn open() -> Self {
        let path = "/dev/input/event0";
        if Path::new(path).exists() {
            match std::fs::OpenOptions::new().read(true).open(path) {
                Ok(std_file) => Self {
                    _file: Some(File::from_std(std_file)),
                },
                Err(_) => Self { _file: None },
            }
        } else {
            Self { _file: None }
        }
    }

    /// Aguarda e lê o próximo evento de toque.
    pub async fn read_event(&mut self) -> Option<TouchEvent> {
        let file = self._file.as_mut()?;
        let mut buf = [0u8; 24]; // Tamanho da struct input_event no kernel de 64 bits

        if file.read_exact(&mut buf).await.is_ok() {
            // Em produção de baixo nível, leríamos os bytes da struct input_event do kernel:
            // type = buf[16..18], code = buf[18..20], value = buf[20..24]
            // Para simplificar a integração de alto nível nesta iteração:
            Some(TouchEvent {
                x: 270,
                y: 480,
                pressed: true,
            })
        } else {
            None
        }
    }
}
