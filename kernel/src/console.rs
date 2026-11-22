use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::fmt;
use font8x8::{BASIC_FONTS, UnicodeFonts};

const GLYPH_WIDTH: usize = 8;
const GLYPH_HEIGHT: usize = 8;
const FOREGROUND: [u8; 3] = [220, 230, 240];
const BACKGROUND: [u8; 3] = [12, 18, 28];

pub struct Console<'a> {
    buffer: &'a mut [u8],
    info: FrameBufferInfo,
    cursor_x: usize,
    cursor_y: usize,
}

impl<'a> Console<'a> {
    pub fn new(buffer: &'a mut [u8], info: FrameBufferInfo) -> Option<Self> {
        if info.width < GLYPH_WIDTH
            || info.height < GLYPH_HEIGHT
            || info.stride < info.width
            || info.bytes_per_pixel == 0
        {
            return None;
        }

        let required_len = info
            .stride
            .checked_mul(info.height)?
            .checked_mul(info.bytes_per_pixel)?;
        if buffer.len() < required_len || info.byte_len < required_len {
            return None;
        }

        match info.pixel_format {
            PixelFormat::Rgb | PixelFormat::Bgr if info.bytes_per_pixel >= 3 => {}
            PixelFormat::U8 if info.bytes_per_pixel >= 1 => {}
            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } if info.bytes_per_pixel <= 4
                && [red_position, green_position, blue_position]
                    .iter()
                    .all(|position| {
                        u16::from(*position) + 8 <= (info.bytes_per_pixel * 8) as u16
                    }) => {}
            _ => return None,
        }

        let mut console = Self {
            buffer,
            info,
            cursor_x: 0,
            cursor_y: 0,
        };
        console.clear();
        Some(console)
    }

    fn clear(&mut self) {
        for y in 0..self.info.height {
            for x in 0..self.info.width {
                self.put_pixel(x, y, BACKGROUND);
            }
        }
    }

    fn write_char(&mut self, character: char) {
        match character {
            '\n' => {
                self.cursor_x = 0;
                self.cursor_y += GLYPH_HEIGHT;
                self.scroll_if_needed();
            }
            '\r' => self.cursor_x = 0,
            character => {
                if self.cursor_x + GLYPH_WIDTH > self.info.width {
                    self.cursor_x = 0;
                    self.cursor_y += GLYPH_HEIGHT;
                    self.scroll_if_needed();
                }

                let glyph = BASIC_FONTS
                    .get(character)
                    .or_else(|| BASIC_FONTS.get('?'))
                    .expect("basic font contains replacement glyph");

                for (row, bits) in glyph.iter().enumerate() {
                    for column in 0..GLYPH_WIDTH {
                        let color = if bits & (1 << column) != 0 {
                            FOREGROUND
                        } else {
                            BACKGROUND
                        };
                        self.put_pixel(self.cursor_x + column, self.cursor_y + row, color);
                    }
                }

                self.cursor_x += GLYPH_WIDTH;
            }
        }
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        let start = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        let pixel = &mut self.buffer[start..start + self.info.bytes_per_pixel];
        pixel.fill(0);

        match self.info.pixel_format {
            PixelFormat::Rgb => pixel[..3].copy_from_slice(&color),
            PixelFormat::Bgr => pixel[..3].copy_from_slice(&[color[2], color[1], color[0]]),
            PixelFormat::U8 => {
                pixel[0] =
                    ((u16::from(color[0]) + u16::from(color[1]) + u16::from(color[2])) / 3) as u8;
            }
            PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => {
                let value = (u32::from(color[0]) << red_position)
                    | (u32::from(color[1]) << green_position)
                    | (u32::from(color[2]) << blue_position);
                for (index, byte) in pixel.iter_mut().enumerate() {
                    *byte = (value >> (index * 8)) as u8;
                }
            }
            _ => {}
        }
    }

    fn scroll_if_needed(&mut self) {
        if self.cursor_y + GLYPH_HEIGHT <= self.info.height {
            return;
        }

        let row_bytes = self.info.stride * self.info.bytes_per_pixel;
        let scroll_bytes = GLYPH_HEIGHT * row_bytes;
        let used_bytes = self.info.height * row_bytes;
        self.buffer.copy_within(scroll_bytes..used_bytes, 0);
        self.buffer[used_bytes - scroll_bytes..used_bytes].fill(0);
        self.cursor_y = self.info.height - GLYPH_HEIGHT;
    }
}

impl fmt::Write for Console<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for character in text.chars() {
            self.write_char(character);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bootloader_api::info::PixelFormat;
    use core::fmt::Write;

    fn info(pixel_format: PixelFormat, bytes_per_pixel: usize) -> FrameBufferInfo {
        FrameBufferInfo {
            byte_len: 16 * 8 * bytes_per_pixel,
            width: 16,
            height: 8,
            pixel_format,
            bytes_per_pixel,
            stride: 16,
        }
    }

    #[test]
    fn renders_rgb_glyphs_and_background() {
        let info = info(PixelFormat::Rgb, 3);
        let mut buffer = [0; 16 * 8 * 3];
        let mut console = Console::new(&mut buffer, info).unwrap();
        write!(console, "A").unwrap();

        assert!(buffer.chunks_exact(3).any(|pixel| pixel == FOREGROUND));
        assert!(buffer.chunks_exact(3).any(|pixel| pixel == BACKGROUND));
    }

    #[test]
    fn renders_bgr_channel_order() {
        let info = info(PixelFormat::Bgr, 3);
        let mut buffer = [0; 16 * 8 * 3];
        let mut console = Console::new(&mut buffer, info).unwrap();
        write!(console, "A").unwrap();

        assert!(
            buffer
                .chunks_exact(3)
                .any(|pixel| pixel == [FOREGROUND[2], FOREGROUND[1], FOREGROUND[0]])
        );
    }

    #[test]
    fn rejects_framebuffers_too_small_for_a_glyph() {
        let info = FrameBufferInfo {
            width: 7,
            ..info(PixelFormat::Rgb, 3)
        };
        let mut buffer = [0; 7 * 8 * 3];

        assert!(Console::new(&mut buffer, info).is_none());
    }
}
