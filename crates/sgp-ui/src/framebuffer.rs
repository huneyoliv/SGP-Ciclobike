//! Gerenciamento de baixo nível do Framebuffer Linux com mmap e double-buffering.

use std::fs::{File, OpenOptions};

use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

/// Abstração sobre o buffer de vídeo físico (/dev/fb0) ou virtual.
pub struct FrameBuffer {
    mmap_ptr: *mut u8,
    width: u32,
    height: u32,
    size: usize,
    back_buffer: Vec<u16>,
    is_virtual: bool,
    _fb_file: Option<File>,
}

impl FrameBuffer {
    /// Abre o framebuffer físico `/dev/fb0`. Caso falhe, retorna um buffer virtual em memória.
    pub fn open() -> Self {
        let width = 540;
        let height = 960;
        let size = (width * height * 2) as usize;
        let back_buffer = vec![0u16; (width * height) as usize];

        match OpenOptions::new().read(true).write(true).open("/dev/fb0") {
            Ok(file) => unsafe {
                let mmap_ptr = nix::sys::mman::mmap(
                    None,
                    std::num::NonZeroUsize::new(size).unwrap(),
                    nix::sys::mman::ProtFlags::PROT_READ | nix::sys::mman::ProtFlags::PROT_WRITE,
                    nix::sys::mman::MapFlags::MAP_SHARED,
                    &file,
                    0,
                );
                match mmap_ptr {
                    Ok(ptr) => Self {
                        mmap_ptr: ptr.as_ptr() as *mut u8,
                        width,
                        height,
                        size,
                        back_buffer,
                        is_virtual: false,
                        _fb_file: Some(file),
                    },
                    Err(_) => Self::new_virtual(width, height, size, back_buffer),
                }
            },
            Err(_) => Self::new_virtual(width, height, size, back_buffer),
        }
    }

    fn new_virtual(width: u32, height: u32, size: usize, back_buffer: Vec<u16>) -> Self {
        Self {
            mmap_ptr: std::ptr::null_mut(),
            width,
            height,
            size,
            back_buffer,
            is_virtual: true,
            _fb_file: None,
        }
    }

    /// Executa o swap de buffers copiando o back buffer em memória para o framebuffer do sistema.
    pub fn flush(&mut self) {
        if !self.is_virtual && !self.mmap_ptr.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.back_buffer.as_ptr() as *const u8,
                    self.mmap_ptr,
                    self.size,
                );
            }
        }
    }

    /// Retorna a largura do display.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Retorna a altura do display.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Retorna o estado do pixel no back buffer (útil para asserções de testes).
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<u16> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            Some(self.back_buffer[idx])
        } else {
            None
        }
    }
}

impl DrawTarget for FrameBuffer {
    type Color = Rgb565;
    type Error = std::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0
                && coord.x < self.width as i32
                && coord.y >= 0
                && coord.y < self.height as i32
            {
                let idx = (coord.y as usize * self.width as usize) + coord.x as usize;
                let raw_u16 = RawU16::from(color).into_inner();
                self.back_buffer[idx] = raw_u16;
            }
        }
        Ok(())
    }
}

impl OriginDimensions for FrameBuffer {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        if !self.is_virtual && !self.mmap_ptr.is_null() {
            unsafe {
                let _ = nix::sys::mman::munmap(
                    std::ptr::NonNull::new(self.mmap_ptr as *mut std::ffi::c_void).unwrap(),
                    self.size,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::pixelcolor::Rgb565;

    #[test]
    fn test_virtual_framebuffer_draw() {
        let mut fb = FrameBuffer::open();
        assert!(fb.is_virtual);
        assert_eq!(fb.width(), 540);
        assert_eq!(fb.height(), 960);

        let pixel = Pixel(Point::new(10, 20), Rgb565::new(31, 0, 0));
        fb.draw_iter(std::iter::once(pixel)).unwrap();

        let raw_pixel = fb.get_pixel(10, 20).unwrap();
        let expected = RawU16::from(Rgb565::new(31, 0, 0)).into_inner();
        assert_eq!(raw_pixel, expected);
    }
}
