use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use simulation::{Board, Brush, Cell};

const BOARD_WIDTH: usize = 300;
const BOARD_HEIGHT: usize = 300;
const SCALE: usize = 3; // each cell drawn as SCALE x SCALE pixels
const WINDOW_WIDTH: usize = BOARD_WIDTH * SCALE;
const WINDOW_HEIGHT: usize = BOARD_HEIGHT * SCALE;

fn main() {
    let mut board = Board::new(BOARD_WIDTH, BOARD_HEIGHT);
    let mut rng = SmallRng::seed_from_u64(0xfa11_5a4d);
    let brush = Brush::new();

    let mut rgba = vec![0u8; BOARD_WIDTH * BOARD_HEIGHT * 4];

    let mut frame = vec![0u32; WINDOW_WIDTH * WINDOW_HEIGHT];

    let mut window = Window::new(
        "Falling Sand",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .expect("failed to open window");

    window.set_target_fps(60);

    let mut current_element = Cell::Sand;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Key1) {
            current_element = Cell::Sand;
        }
        if window.is_key_down(Key::Key2) {
            current_element = Cell::Water;
        }
        if window.is_key_down(Key::Key3) {
            current_element = Cell::Stone;
        }
        if window.is_key_down(Key::Key0) {
            current_element = Cell::Empty;
        }

        if window.get_mouse_down(MouseButton::Left)
            && let Some((mx, my)) = window.get_mouse_pos(MouseMode::Discard)
        {
            let cx = (mx as usize) / SCALE;
            let cy = (my as usize) / SCALE;
            brush.paint(&mut board, cx, cy, current_element, &mut rng);
        }

        board.step(&mut rng);
        board.render(&mut rgba);

        for y in 0..WINDOW_HEIGHT {
            let src_y = y / SCALE;
            for x in 0..WINDOW_WIDTH {
                let src_x = x / SCALE;
                let src_idx = (src_y * BOARD_WIDTH + src_x) * 4;
                let r = rgba[src_idx] as u32;
                let g = rgba[src_idx + 1] as u32;
                let b = rgba[src_idx + 2] as u32;
                frame[y * WINDOW_WIDTH + x] = (r << 16) | (g << 8) | b;
            }
        }

        window
            .update_with_buffer(&frame, WINDOW_WIDTH, WINDOW_HEIGHT)
            .expect("failed to update window");
    }
}
