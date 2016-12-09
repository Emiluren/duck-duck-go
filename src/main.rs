extern crate sfml;

use sfml::graphics::{RenderWindow, Color, RenderTarget};
use sfml::window::{VideoMode, ContextSettings, event, window_style};

fn main() {
    let game_width = 800;
    let game_height = 600;

    let mut window = RenderWindow::new(
        VideoMode::new_init(game_width, game_height, 32),
        "Duck duck swim",
        window_style::CLOSE,
        &ContextSettings::default()
    ).expect("Failed to create window.");

    while window.is_open() {
        for event in window.events() {
            match event {
                event::Closed => window.close(),
                _ => {}
            }
        }
        window.clear(&Color::new_rgb(0, 0, 0));
        window.display();
    }
}
