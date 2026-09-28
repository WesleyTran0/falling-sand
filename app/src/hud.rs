//! On-screen HUD: a baked bitmap font, alpha-blended drawing primitives, and
//! the controls legend the app overlays on the finished frame.
//!
//! This module belongs to `app`, not to `simulation`: it draws into the window
//! framebuffer (`0RGB` `u32` pixels, row-major) and knows nothing about the
//! grid beyond which `Cell` is currently selected.
//!
//! Geometry is always supplied by the caller. Nothing here hardcodes where the
//! panel lives, so moving the HUD out of the board area (into a dedicated strip
//! below it) is a change of arguments in `main`, not a change here.

use simulation::{Cell, cell_color};

/// Width of a glyph cell, in unscaled font pixels.
pub const GLYPH_WIDTH: usize = 5;
/// Height of a glyph cell, in unscaled font pixels.
pub const GLYPH_HEIGHT: usize = 7;
/// Blank columns between adjacent glyphs, in unscaled font pixels.
pub const GLYPH_SPACING: usize = 1;

/// One glyph: `GLYPH_HEIGHT` rows, top row first (the framebuffer is y-down,
/// and so is the font). In each row, bit `GLYPH_WIDTH - 1` is the leftmost
/// pixel and bit `0` the rightmost, so the binary literals below read as
/// pictures of the character.
type Glyph = [u8; GLYPH_HEIGHT];

/// Returns the bitmap for `c`, or `None` if the baked font has no glyph for it.
///
/// Lowercase input is folded to uppercase; the font is uppercase-only.
/// Callers must not assume every `char` is drawable — `draw_text` skips
/// unknown characters, and `hud_strings_are_all_drawable` keeps the strings
/// this module actually draws inside the supported set.
pub fn glyph(c: char) -> Option<Glyph> {
    let g: Glyph = match c.to_ascii_uppercase() {
        ' ' => [0; GLYPH_HEIGHT],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10011, 0b10011, 0b10101, 0b11001, 0b11001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b00100, 0b01000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        '=' => [
            0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000,
        ],
        '/' => [
            0b00001, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        '?' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
        '\'' => [
            0b00100, 0b00100, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        _ => return None,
    };
    Some(g)
}

/// Width in pixels that `text` occupies when drawn at `scale`.
///
/// Includes `GLYPH_SPACING` between glyphs but not after the last one.
/// Returns `0` for the empty string.
pub fn text_width(text: &str, scale: usize) -> usize {
    let count = text.chars().count();
    if count == 0 {
        return 0;
    }
    (count * (GLYPH_WIDTH + GLYPH_SPACING) - GLYPH_SPACING) * scale
}

/// Height in pixels of one line of text drawn at `scale`.
pub const fn text_height(scale: usize) -> usize {
    GLYPH_HEIGHT * scale
}

/// An axis-aligned rectangle in framebuffer pixels. `x`/`y` are the top-left
/// corner; the framebuffer is y-down, so `y` grows toward the bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// A mutable view over the window framebuffer: `0RGB` `u32` pixels, row-major,
/// `width * height` of them.
///
/// Every drawing method clips against the view's bounds and silently ignores
/// out-of-range pixels rather than panicking or wrapping onto the next row.
pub struct Frame<'a> {
    pixels: &'a mut [u32],
    width: usize,
    height: usize,
}

impl<'a> Frame<'a> {
    /// Wraps `pixels` as a `width * height` framebuffer.
    ///
    /// Returns `None` if `pixels.len()` is not exactly `width * height`.
    pub fn new(pixels: &'a mut [u32], width: usize, height: usize) -> Option<Self> {
        if pixels.len() != width * height {
            return None;
        }
        Some(Self {
            pixels,
            width,
            height,
        })
    }

    /// Blends `rgb` over the pixel at `(x, y)` with coverage `alpha`
    /// (`0` = leave unchanged, `255` = fully replace).
    ///
    /// Out-of-bounds coordinates are ignored, so text never wraps onto the
    /// next row and never panics.
    pub fn blend_pixel(&mut self, x: usize, y: usize, rgb: u32, alpha: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * self.width + x;
        self.pixels[idx] = blend(self.pixels[idx], rgb, alpha);
    }

    /// Blends `rgb` over every pixel of `rect` with coverage `alpha`.
    /// The parts of `rect` outside the framebuffer are clipped away.
    pub fn fill_rect(&mut self, rect: Rect, rgb: u32, alpha: u8) {
        let x_end = rect.x.saturating_add(rect.width).min(self.width);
        let y_end = rect.y.saturating_add(rect.height).min(self.height);
        for y in rect.y..y_end {
            for x in rect.x..x_end {
                self.blend_pixel(x, y, rgb, alpha);
            }
        }
    }

    /// Draws `text` opaquely in `rgb` with its top-left corner at `(x, y)`,
    /// each font pixel expanded to a `scale` x `scale` block.
    ///
    /// Characters with no glyph advance the pen but draw nothing. Pixels
    /// outside the framebuffer are clipped. Returns the pen x position just
    /// past the last glyph, so callers can append more content in the same
    /// line (this advance ignores clipping and may exceed the frame width).
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, rgb: u32, scale: usize) -> usize {
        let mut pen_x = x;
        for c in text.chars() {
            if let Some(g) = glyph(c) {
                for (row, bits) in g.iter().enumerate() {
                    for col in 0..GLYPH_WIDTH {
                        if bits & (1 << (GLYPH_WIDTH - 1 - col)) == 0 {
                            continue;
                        }
                        self.fill_rect(
                            Rect {
                                x: pen_x + col * scale,
                                y: y + row * scale,
                                width: scale,
                                height: scale,
                            },
                            rgb,
                            0xff,
                        );
                    }
                }
            }
            pen_x += (GLYPH_WIDTH + GLYPH_SPACING) * scale;
        }
        pen_x.saturating_sub(GLYPH_SPACING * scale)
    }
}

/// Blends `src` over `dst` (both `0RGB`) with coverage `alpha`, per channel.
///
/// Each channel is computed in `u32` and rounded, so the result is always in
/// `0..=255` and can never wrap around.
fn blend(dst: u32, src: u32, alpha: u8) -> u32 {
    let a = alpha as u32;
    let inv = 255 - a;
    let mut out = 0u32;
    for shift in [16, 8, 0] {
        let d = (dst >> shift) & 0xff;
        let s = (src >> shift) & 0xff;
        let c = (s * a + d * inv + 127) / 255;
        out |= c << shift;
    }
    out
}

/// The fixed help lines of the legend, top to bottom. The selected-element
/// line is appended below these by `draw`.
pub const HELP_LINES: [&str; 2] = [
    "1 SAND   2 WATER   3 STONE   0 ERASE",
    "LMB PAINT   H TOGGLE HUD   ESC QUIT",
];

/// Label in front of the current element on the last HUD line.
pub const SELECTED_PREFIX: &str = "SELECTED: ";

/// Total number of text lines the HUD draws.
pub const LINE_COUNT: usize = HELP_LINES.len() + 1;

/// The HUD's name for `cell`, as shown on the selected-element line.
pub fn element_label(cell: Cell) -> &'static str {
    match cell {
        Cell::Sand => "SAND",
        Cell::Water => "WATER",
        Cell::Stone => "STONE",
        Cell::Empty => "ERASE",
    }
}

/// The swatch colour drawn next to the selected element, as `0RGB`.
///
/// Derived from `simulation`'s `cell_color` so the swatch cannot drift from
/// what the board actually renders. `Cell::Empty` is the exception: its sim
/// colour is the near-black background, which would be invisible against the
/// panel, so the eraser gets a neutral grey.
pub fn element_swatch(cell: Cell) -> u32 {
    if cell == Cell::Empty {
        return 0x0030_3038;
    }
    let [r, g, b, _] = cell_color(cell);
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

/// Visual tuning for the legend panel. Sizes are in framebuffer pixels;
/// colours are `0RGB`.
#[derive(Clone, Copy, Debug)]
pub struct HudStyle {
    /// Integer pixel scale of the font. Keeps glyphs crisp (no filtering).
    pub scale: usize,
    /// Inset between the panel edge and the text block.
    pub padding: usize,
    /// Vertical gap between text lines.
    pub line_gap: usize,
    /// Panel fill colour.
    pub panel_rgb: u32,
    /// Panel coverage over the pixels underneath: `255` is opaque.
    pub panel_alpha: u8,
    /// Text colour.
    pub text_rgb: u32,
}

impl Default for HudStyle {
    fn default() -> Self {
        Self {
            scale: 3,
            padding: 8,
            line_gap: 4,
            panel_rgb: 0x0000_0008,
            // Dark enough to keep white text legible over a bright sand pile,
            // sheer enough to keep the sim visible underneath.
            panel_alpha: 0xc4,
            text_rgb: 0x00ff_ffff,
        }
    }
}

impl HudStyle {
    /// Height in pixels of a panel holding `LINE_COUNT` lines at this style.
    pub const fn panel_height(&self) -> usize {
        self.padding * 2 + LINE_COUNT * text_height(self.scale) + (LINE_COUNT - 1) * self.line_gap
    }
}

/// Draws the legend panel with its top-left corner at `(origin_x, origin_y)`
/// and the given `width`, and marks `selected` as the active element.
///
/// The caller owns the geometry: pass the top of the board to overlay the HUD
/// on the sim, or the top of a dedicated strip below the board (with
/// `panel_alpha` at `0xff`) to sit outside it. Anything that falls outside the
/// frame is clipped.
pub fn draw(
    frame: &mut Frame,
    origin_x: usize,
    origin_y: usize,
    width: usize,
    style: &HudStyle,
    selected: Cell,
) {
    frame.fill_rect(
        Rect {
            x: origin_x,
            y: origin_y,
            width,
            height: style.panel_height(),
        },
        style.panel_rgb,
        style.panel_alpha,
    );

    let line_step = text_height(style.scale) + style.line_gap;
    let text_x = origin_x + style.padding;
    let mut text_y = origin_y + style.padding;

    for line in HELP_LINES {
        frame.draw_text(text_x, text_y, line, style.text_rgb, style.scale);
        text_y += line_step;
    }

    let pen = frame.draw_text(text_x, text_y, SELECTED_PREFIX, style.text_rgb, style.scale);
    let pen = frame.draw_text(
        pen + GLYPH_SPACING * style.scale,
        text_y,
        element_label(selected),
        style.text_rgb,
        style.scale,
    );
    let swatch = text_height(style.scale);
    frame.fill_rect(
        Rect {
            x: pen + 2 * GLYPH_SPACING * style.scale,
            y: text_y,
            width: swatch,
            height: swatch,
        },
        element_swatch(selected),
        0xff,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only read access. `Frame` borrows the framebuffer mutably, so a
    /// test cannot inspect the underlying slice while the frame is alive.
    impl Frame<'_> {
        fn pixel(&self, x: usize, y: usize) -> Option<u32> {
            if x >= self.width || y >= self.height {
                return None;
            }
            Some(self.pixels[y * self.width + x])
        }

        fn width(&self) -> usize {
            self.width
        }
    }

    /// Builds a `width` x `height` scratch framebuffer filled with `fill`.
    fn scratch(width: usize, height: usize, fill: u32) -> Vec<u32> {
        vec![fill; width * height]
    }

    #[test]
    fn frame_new_rejects_a_mismatched_buffer_length() {
        let mut pixels = scratch(4, 4, 0);
        assert!(Frame::new(&mut pixels, 4, 5).is_none());
        assert!(Frame::new(&mut pixels, 4, 4).is_some());
    }

    #[test]
    fn hud_strings_are_all_drawable() {
        let mut strings: Vec<&str> = HELP_LINES.to_vec();
        strings.push(SELECTED_PREFIX);
        for cell in [Cell::Sand, Cell::Water, Cell::Stone, Cell::Empty] {
            strings.push(element_label(cell));
        }

        for s in strings {
            for c in s.chars() {
                assert!(
                    glyph(c).is_some(),
                    "HUD string {s:?} uses {c:?}, which the baked font has no glyph for"
                );
            }
        }
    }

    #[test]
    fn text_width_counts_glyphs_and_inter_glyph_spacing() {
        assert_eq!(text_width("", 3), 0);
        assert_eq!(text_width("A", 1), GLYPH_WIDTH);
        assert_eq!(text_width("A", 3), GLYPH_WIDTH * 3);
        assert_eq!(text_width("AB", 1), GLYPH_WIDTH * 2 + GLYPH_SPACING);
    }

    #[test]
    fn draw_text_is_not_flipped_or_mirrored() {
        // 'L' is asymmetric both horizontally and vertically: a full left
        // column and a full bottom row. Drawn at scale 1 with the origin at
        // (0, 0) it pins the font to the framebuffer's y-down, x-right axes.
        let mut pixels = scratch(16, 16, 0x0000_0000);
        let mut frame = Frame::new(&mut pixels, 16, 16).unwrap();
        frame.draw_text(0, 0, "L", 0x00ff_ffff, 1);

        // Left column lit, top to bottom.
        for y in 0..GLYPH_HEIGHT {
            assert_eq!(frame.pixel(0, y), Some(0x00ff_ffff), "left column row {y}");
        }
        // Bottom row lit, left to right.
        for x in 0..GLYPH_WIDTH {
            let y = GLYPH_HEIGHT - 1;
            assert_eq!(frame.pixel(x, y), Some(0x00ff_ffff), "bottom row col {x}");
        }
        // The top-right corner is empty: an upside-down or mirrored 'L'
        // would light it.
        assert_eq!(frame.pixel(GLYPH_WIDTH - 1, 0), Some(0));
        assert_eq!(frame.pixel(GLYPH_WIDTH - 1, 1), Some(0));
        // Nothing spills past the glyph cell.
        assert_eq!(frame.pixel(GLYPH_WIDTH, 0), Some(0));
        assert_eq!(frame.pixel(0, GLYPH_HEIGHT), Some(0));
    }

    #[test]
    fn draw_text_scale_expands_each_font_pixel_into_a_block() {
        let mut pixels = scratch(16, 16, 0);
        let mut frame = Frame::new(&mut pixels, 16, 16).unwrap();
        frame.draw_text(0, 0, "L", 0x00ff_ffff, 2);

        // The top-left font pixel becomes a 2x2 block.
        for y in 0..2 {
            for x in 0..2 {
                assert_eq!(frame.pixel(x, y), Some(0x00ff_ffff), "block ({x},{y})");
            }
        }
        // The column to its right is still background.
        assert_eq!(frame.pixel(2, 0), Some(0));
    }

    #[test]
    fn draw_text_advance_matches_text_width() {
        let mut pixels = scratch(200, 16, 0);
        let mut frame = Frame::new(&mut pixels, 200, 16).unwrap();
        let pen = frame.draw_text(10, 0, "ABC", 0x00ff_ffff, 2);
        assert_eq!(pen, 10 + text_width("ABC", 2));
    }

    #[test]
    fn unknown_characters_advance_the_pen_without_drawing() {
        // `~` has no glyph; the pen must still advance so the rest of the
        // line stays aligned.
        let mut pixels = scratch(64, 16, 0);
        let mut frame = Frame::new(&mut pixels, 64, 16).unwrap();
        assert!(glyph('~').is_none());
        let pen = frame.draw_text(0, 0, "~", 0x00ff_ffff, 1);
        assert_eq!(pen, text_width("~", 1));
        assert!(
            pixels.iter().all(|&p| p == 0),
            "an unsupported character must not draw anything"
        );
    }

    #[test]
    fn blend_over_a_solid_colour_produces_the_expected_value() {
        // 50% (128/255) of black over mid-grey 0x808080.
        let blended = blend(0x0080_8080, 0x0000_0000, 128);
        let expected = (0x80 * 127 + 127) / 255; // 64
        assert_eq!(expected, 64);
        assert_eq!(blended, (expected << 16) | (expected << 8) | expected);
    }

    #[test]
    fn blend_endpoints_are_exact() {
        assert_eq!(blend(0x0012_3456, 0x00ab_cdef, 0), 0x0012_3456);
        assert_eq!(blend(0x0012_3456, 0x00ab_cdef, 255), 0x00ab_cdef);
    }

    #[test]
    fn blend_never_overflows_a_channel() {
        for alpha in 0..=255u8 {
            for &(dst, src) in &[
                (0x00ff_ffff, 0x00ff_ffff),
                (0x00ff_ffff, 0x0000_0000),
                (0x0000_0000, 0x00ff_ffff),
                (0x00ff_00ff, 0x0000_ff00),
            ] {
                let out = blend(dst, src, alpha);
                assert_eq!(out & 0xff00_0000, 0, "alpha byte must stay clear");
                for shift in [16, 8, 0] {
                    let c = (out >> shift) & 0xff;
                    let lo = ((dst >> shift) & 0xff).min((src >> shift) & 0xff);
                    let hi = ((dst >> shift) & 0xff).max((src >> shift) & 0xff);
                    assert!(
                        (lo..=hi).contains(&c),
                        "channel {c} outside {lo}..={hi} (alpha {alpha})"
                    );
                }
            }
        }
    }

    #[test]
    fn fill_rect_blends_the_panel_over_known_pixels() {
        let mut pixels = scratch(4, 4, 0x00ff_ffff);
        let mut frame = Frame::new(&mut pixels, 4, 4).unwrap();
        frame.fill_rect(
            Rect {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            },
            0x0000_0000,
            128,
        );
        let blended = blend(0x00ff_ffff, 0x0000_0000, 128);
        assert_eq!(frame.pixel(1, 1), Some(blended));
        assert_eq!(frame.pixel(2, 2), Some(blended));
        // Outside the rect is untouched.
        assert_eq!(frame.pixel(0, 0), Some(0x00ff_ffff));
        assert_eq!(frame.pixel(3, 3), Some(0x00ff_ffff));
    }

    #[test]
    fn fill_rect_clips_instead_of_panicking() {
        let mut pixels = scratch(4, 4, 0);
        let mut frame = Frame::new(&mut pixels, 4, 4).unwrap();
        frame.fill_rect(
            Rect {
                x: 3,
                y: 3,
                width: usize::MAX,
                height: usize::MAX,
            },
            0x00ff_ffff,
            0xff,
        );
        assert_eq!(frame.pixel(3, 3), Some(0x00ff_ffff));

        // Entirely off the right/bottom edge: nothing changes.
        frame.fill_rect(
            Rect {
                x: 10,
                y: 10,
                width: 4,
                height: 4,
            },
            0x00ff_0000,
            0xff,
        );
        assert_eq!(pixels.iter().filter(|&&p| p != 0).count(), 1);
    }

    #[test]
    fn text_past_the_right_edge_is_clipped_and_never_wraps() {
        // 'I' has a full-width top row and a single centre pixel on row 1.
        // Drawn two pixels from the right edge, the centre pixel of row 1
        // lands off-screen: if drawing wrapped, it would appear in row 2.
        let width = 8;
        let height = 8;
        let mut pixels = scratch(width, height, 0);
        let mut frame = Frame::new(&mut pixels, width, height).unwrap();
        frame.draw_text(width - 2, 0, "I", 0x00ff_ffff, 1);

        // The visible sliver of the top row is drawn...
        assert_eq!(frame.pixel(width - 2, 0), Some(0x00ff_ffff));
        assert_eq!(frame.pixel(width - 1, 0), Some(0x00ff_ffff));
        // ...and every lit pixel is at or right of the pen: nothing wrapped
        // around onto the next row.
        for y in 0..height {
            for x in 0..width - 2 {
                assert_eq!(frame.pixel(x, y), Some(0), "wrapped pixel at ({x},{y})");
            }
        }
        // Rows 1..=5 of 'I' are a single centre pixel, which is off-screen
        // here, so they must be entirely blank.
        for y in 1..GLYPH_HEIGHT - 1 {
            for x in 0..width {
                assert_eq!(frame.pixel(x, y), Some(0), "stray pixel at ({x},{y})");
            }
        }
    }

    #[test]
    fn text_past_the_bottom_edge_is_clipped_and_never_panics() {
        let width = 8;
        let height = 3;
        let mut pixels = scratch(width, height, 0);
        let mut frame = Frame::new(&mut pixels, width, height).unwrap();
        // Origin on the last row: only the glyph's top row is visible.
        frame.draw_text(0, height - 1, "I", 0x00ff_ffff, 1);
        for x in 0..GLYPH_WIDTH {
            assert_eq!(frame.pixel(x, height - 1), Some(0x00ff_ffff));
        }
        for y in 0..height - 1 {
            for x in 0..width {
                assert_eq!(frame.pixel(x, y), Some(0), "stray pixel at ({x},{y})");
            }
        }

        // Origin fully below the frame: nothing drawn, no panic.
        let mut pixels = scratch(width, height, 0);
        let mut frame = Frame::new(&mut pixels, width, height).unwrap();
        frame.draw_text(0, height + 5, "SAND", 0x00ff_ffff, 1);
        assert!(pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn panel_height_covers_every_line_plus_padding() {
        let style = HudStyle::default();
        let expected = style.padding * 2
            + LINE_COUNT * GLYPH_HEIGHT * style.scale
            + (LINE_COUNT - 1) * style.line_gap;
        assert_eq!(style.panel_height(), expected);
        assert!(style.panel_height() > LINE_COUNT * GLYPH_HEIGHT * style.scale);
    }

    #[test]
    fn draw_covers_the_panel_rect_and_leaves_the_rest_of_the_frame_alone() {
        let style = HudStyle::default();
        let width = text_width(HELP_LINES[0], style.scale) + 4 * style.padding;
        let height = style.panel_height() * 3;
        let mut pixels = scratch(width, height, 0x0010_1018);
        let mut frame = Frame::new(&mut pixels, width, height).unwrap();
        hud_draw_at_origin(&mut frame, 0, style, Cell::Water);

        // The panel darkened the top band...
        assert_ne!(frame.pixel(0, 0), Some(0x0010_1018));
        // ...and text was drawn somewhere inside it.
        let lit = (0..style.panel_height())
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .filter(|&(x, y)| frame.pixel(x, y) == Some(style.text_rgb))
            .count();
        assert!(lit > 0, "no text pixels inside the panel");
        // ...but everything below the panel is untouched.
        for y in style.panel_height()..height {
            for x in 0..width {
                assert_eq!(frame.pixel(x, y), Some(0x0010_1018), "leak at ({x},{y})");
            }
        }
    }

    #[test]
    fn draw_at_a_lower_origin_moves_the_whole_panel() {
        // Stage two relocates the HUD by changing its origin only; nothing
        // above the new origin may be touched.
        let style = HudStyle::default();
        let width = text_width(HELP_LINES[0], style.scale) + 4 * style.padding;
        let origin_y = 20;
        let height = origin_y + style.panel_height();
        let mut pixels = scratch(width, height, 0x0010_1018);
        let mut frame = Frame::new(&mut pixels, width, height).unwrap();
        hud_draw_at_origin(&mut frame, origin_y, style, Cell::Sand);

        for y in 0..origin_y {
            for x in 0..width {
                assert_eq!(frame.pixel(x, y), Some(0x0010_1018), "leak at ({x},{y})");
            }
        }
        assert_ne!(frame.pixel(0, origin_y), Some(0x0010_1018));
    }

    #[test]
    fn draw_into_a_frame_smaller_than_the_panel_does_not_panic() {
        let style = HudStyle::default();
        let mut pixels = scratch(10, 6, 0);
        let mut frame = Frame::new(&mut pixels, 10, 6).unwrap();
        hud_draw_at_origin(&mut frame, 0, style, Cell::Stone);
    }

    fn hud_draw_at_origin(frame: &mut Frame, origin_y: usize, style: HudStyle, selected: Cell) {
        let width = frame.width();
        draw(frame, 0, origin_y, width, &style, selected);
    }

    #[test]
    fn element_swatch_matches_the_sim_palette() {
        for cell in [Cell::Sand, Cell::Water, Cell::Stone] {
            let [r, g, b, _] = cell_color(cell);
            let expected = ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            assert_eq!(
                element_swatch(cell),
                expected,
                "swatch for {cell:?} drifted from the board's own colour"
            );
        }
    }

    #[test]
    fn eraser_swatch_is_not_the_invisible_background() {
        // `Cell::Empty`'s sim colour is the near-black background. Drawing the
        // eraser swatch in it would make the swatch disappear into the panel,
        // so it is deliberately the one colour that does not match.
        let [r, g, b, _] = cell_color(Cell::Empty);
        let sim = ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
        assert_ne!(element_swatch(Cell::Empty), sim);
    }
}
