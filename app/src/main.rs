mod hud;

use hud::{Frame, HudStyle};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use simulation::{Board, Brush, Cell};

const BOARD_WIDTH: usize = 300;
const BOARD_HEIGHT: usize = 300;
const SCALE: usize = 3; // each cell drawn as SCALE x SCALE pixels

/// The board occupies this rectangle of the window, top-left aligned.
const BOARD_PIXEL_WIDTH: usize = BOARD_WIDTH * SCALE;
const BOARD_PIXEL_HEIGHT: usize = BOARD_HEIGHT * SCALE;

/// Extra window height below the board reserved for the HUD.
///
/// Stage one: `0` — the HUD is translucent and overlays the top of the board.
/// Stage two (HUD below the board): set this to `HUD_STYLE.panel_height()`,
/// set `HUD_ORIGIN_Y` to `BOARD_PIXEL_HEIGHT` and `HUD_STYLE.panel_alpha` to
/// `0xff`. The window grows, the board does not, and nothing else moves.
const HUD_STRIP_HEIGHT: usize = 0;

const WINDOW_WIDTH: usize = BOARD_PIXEL_WIDTH;
const WINDOW_HEIGHT: usize = BOARD_PIXEL_HEIGHT + HUD_STRIP_HEIGHT;

/// Top-left corner and width of the HUD panel, in window pixels.
const HUD_ORIGIN_X: usize = 0;
const HUD_ORIGIN_Y: usize = 0;
const HUD_WIDTH: usize = WINDOW_WIDTH;

/// Edge-triggered key that shows/hides the HUD (it covers part of the board).
const HUD_TOGGLE_KEY: Key = Key::H;

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
    let hud_style = HudStyle::default();
    let mut hud_visible = true;

    // The legend is only useful if it fits the panel; catch an over-long
    // line (or a too-large text scale) while developing rather than by
    // noticing clipped text in the window.
    for line in hud::HELP_LINES {
        debug_assert!(
            hud::text_width(line, hud_style.scale) + 2 * hud_style.padding <= HUD_WIDTH,
            "HUD line {line:?} is too wide for the panel"
        );
    }

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

        // Edge-triggered: `is_key_down` would flip the HUD every frame.
        if window
            .get_keys_pressed(KeyRepeat::No)
            .contains(&HUD_TOGGLE_KEY)
        {
            hud_visible = !hud_visible;
        }

        if window.get_mouse_down(MouseButton::Left)
            && let Some((mx, my)) = window.get_mouse_pos(MouseMode::Discard)
        {
            let cx = (mx as usize) / SCALE;
            let cy = (my as usize) / SCALE;
            // Clicks outside the board (e.g. a future HUD strip below it) are
            // dropped by `Board::set`'s bounds check.
            brush.paint(&mut board, cx, cy, current_element, &mut rng);
        }

        board.step(&mut rng);
        board.render(&mut rgba);

        for y in 0..BOARD_PIXEL_HEIGHT {
            let src_y = y / SCALE;
            for x in 0..BOARD_PIXEL_WIDTH {
                let src_x = x / SCALE;
                let src_idx = (src_y * BOARD_WIDTH + src_x) * 4;
                let r = rgba[src_idx] as u32;
                let g = rgba[src_idx + 1] as u32;
                let b = rgba[src_idx + 2] as u32;
                frame[y * WINDOW_WIDTH + x] = (r << 16) | (g << 8) | b;
            }
        }

        if hud_visible && let Some(mut view) = Frame::new(&mut frame, WINDOW_WIDTH, WINDOW_HEIGHT) {
            hud::draw(
                &mut view,
                HUD_ORIGIN_X,
                HUD_ORIGIN_Y,
                HUD_WIDTH,
                &hud_style,
                current_element,
            );
        }

        window
            .update_with_buffer(&frame, WINDOW_WIDTH, WINDOW_HEIGHT)
            .expect("failed to update window");
    }
}
