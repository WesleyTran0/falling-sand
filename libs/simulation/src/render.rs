use crate::board::Board;
use crate::cell::Cell;

pub fn cell_color(cell: Cell) -> [u8; 4] {
    match cell {
        Cell::Empty => [0x10, 0x10, 0x18, 0xff],
        Cell::Sand => [0xe6, 0xc8, 0x7a, 0xff],
        Cell::Water => [0x4a, 0x90, 0xe2, 0xff],
        Cell::Stone => [0x70, 0x70, 0x78, 0xff],
    }
}

impl Board {
    /// Writes the board state into `buffer` as RGBA8.
    /// The buffer must be exactly `width * height * 4` bytes long.
    pub fn render(&self, buffer: &mut [u8]) {
        assert_eq!(
            buffer.len(),
            self.width() * self.height() * 4,
            "render buffer is the wrong size"
        );

        for y in 0..self.height() {
            for x in 0..self.width() {
                let cell = self.get(x, y).unwrap();
                let color = cell_color(cell);
                let pixel_idx = (y * self.width() + x) * 4;
                buffer[pixel_idx..pixel_idx + 4].copy_from_slice(&color);
            }
        }
    }
}
