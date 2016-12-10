extern crate sfml;
extern crate specs;

use std::sync::{Arc, RwLock, Mutex};
use sfml::graphics::{RenderWindow, Color, RenderTarget, Sprite};
use sfml::window::{VideoMode, ContextSettings, event, window_style};

struct CompSprite<'a> {
    sprite: Arc<Mutex<Sprite<'a>>>
}

impl<'a> specs::Component for CompSprite<'a> {
    type Storage = specs::VecStorage<CompSprite<'a>>;
}

fn main() {
    let game_width = 800;
    let game_height = 600;

    let mut window = RenderWindow::new(
        VideoMode::new_init(game_width, game_height, 32),
        "Duck duck swim",
        window_style::CLOSE,
        &ContextSettings::default()
    ).expect("Failed to create window.");

    let mut planner = {
        let mut w = specs::World::new();

        w.register::<Sprite>();

        let mut duck_sprite = Sprite::new();
        //duck_sprite.set_texture();

        w.create_now().with(duck_sprite).build();

        specs::Planner::<()>::new(w, 4);
    };

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
