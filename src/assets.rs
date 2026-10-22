//! Built-in bitmap assets, so the compositor has no font or image files to ship.

pub type Rgba = [u8; 4];

pub const OCEAN_BLUE: Rgba = [0x1E, 0x88, 0xC7, 0xFF];
pub const WHITE: Rgba = [0xFF, 0xFF, 0xFF, 0xFF];
pub const BLACK: Rgba = [0x00, 0x00, 0x00, 0xFF];

/// A tightly packed BGRA8888 image, the layout smithay's memory buffers expect.
pub struct Bitmap {
    pub pixels: Vec<u8>,
    pub width: i32,
    pub height: i32,
}

const GLYPH_W: usize = 5;
const GLYPH_H: usize = 7;

/// 5x7 glyphs, one row per entry, most significant bit on the left.
fn glyph(c: char) -> [u8; GLYPH_H] {
    match c {
        'h' => [0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001],
        'e' => [0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110],
        'l' => [0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'o' => [0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
        'w' => [0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010],
        'r' => [0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000],
        'd' => [0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111],
        _ => [0; GLYPH_H],
    }
}

fn put(pixels: &mut [u8], width: usize, x: usize, y: usize, color: Rgba) {
    let i = (y * width + x) * 4;
    // BGRA byte order
    pixels[i] = color[2];
    pixels[i + 1] = color[1];
    pixels[i + 2] = color[0];
    pixels[i + 3] = color[3];
}

/// Renders `text` with the built-in 5x7 font, each font pixel becoming a `scale` x `scale` block.
pub fn text(text: &str, scale: usize, color: Rgba) -> Bitmap {
    let count = text.chars().count();
    let width = count * (GLYPH_W + 1) * scale;
    let height = GLYPH_H * scale;
    let mut pixels = vec![0u8; width * height * 4];

    for (index, c) in text.chars().enumerate() {
        let origin_x = index * (GLYPH_W + 1) * scale;
        for (row, bits) in glyph(c).iter().enumerate() {
            for col in 0..GLYPH_W {
                if bits & (1 << (GLYPH_W - 1 - col)) == 0 {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = origin_x + col * scale + dx;
                        let y = row * scale + dy;
                        put(&mut pixels, width, x, y, color);
                    }
                }
            }
        }
    }

    Bitmap {
        pixels,
        width: width as i32,
        height: height as i32,
    }
}

/// Classic arrow pointer. `#` is fill, `o` is outline, anything else is transparent.
const CURSOR_ROWS: [&str; 17] = [
    "o          ",
    "oo         ",
    "o#o        ",
    "o##o       ",
    "o###o      ",
    "o####o     ",
    "o#####o    ",
    "o######o   ",
    "o#######o  ",
    "o########o ",
    "o#####ooooo",
    "o##o##o    ",
    "o#o  o##o  ",
    "oo    o##o ",
    "o      o##o",
    "       o##o",
    "        oo ",
];

pub fn cursor(scale: usize) -> Bitmap {
    let cols = CURSOR_ROWS[0].len();
    let width = cols * scale;
    let height = CURSOR_ROWS.len() * scale;
    let mut pixels = vec![0u8; width * height * 4];

    for (row, line) in CURSOR_ROWS.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let color = match ch {
                '#' => WHITE,
                'o' => BLACK,
                _ => continue,
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    put(&mut pixels, width, col * scale + dx, row * scale + dy, color);
                }
            }
        }
    }

    Bitmap {
        pixels,
        width: width as i32,
        height: height as i32,
    }
}
