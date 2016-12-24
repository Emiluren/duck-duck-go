extern crate sfml;
extern crate specs;
extern crate nalgebra as na;

use std::sync::{Arc, RwLock, Mutex};
use std::sync::mpsc::channel;
use sfml::graphics::{RenderWindow, Color, RenderTarget, Sprite, Texture};
use sfml::window::{VideoMode, ContextSettings, event, window_style};
use na::Vector2;

#[derive(Clone, Copy)]
struct Transform {
    pos: Vector2<f64>,
}

impl specs::Component for Transform {
    type Storage = specs::VecStorage<Transform>;
}

type SpriteRef = usize;

#[derive(Clone, Copy)]
struct Renderable {
    spriteRef: SpriteRef,
}

impl specs::Component for Renderable {
    type Storage = specs::VecStorage<Renderable>;
}

#[derive(Clone, Copy)]
struct RenderData {
    renderable: Renderable,
    transform: Transform,
}

fn main() {
    let game_width = 800;
    let game_height = 600;

    let mut window = RenderWindow::new(VideoMode::new_init(game_width, game_height, 32),
                                       "Duck duck swim",
                                       window_style::CLOSE,
                                       &ContextSettings::default())
        .expect("Failed to create window.");

    let duck_texture = Texture::new_from_file("assets/duck1.png").unwrap();
    let mut duck_sprite = Sprite::new_with_texture(&duck_texture).unwrap();

    let mut planner = {
        let mut w = specs::World::new();

        w.register::<Transform>();
        w.register::<Renderable>();

        let transform = Transform { pos: Vector2::new(0.0, 0.0) };
        let renderable = Renderable { spriteRef: 0 };
        w.create_now().with(transform).with(renderable).build();

        specs::Planner::<()>::new(w, 4)
    };

    let (tx, rx) = channel();
    while window.is_open() {
        for event in window.events() {
            match event {
                event::Closed => window.close(),
                _ => {}
            }
        }
        window.clear(&Color::new_rgb(0, 0, 0));
        planner.run0w2r(|transform: &Transform, renderable: &Renderable| {
            let thread_tx = tx.clone();
            tx.send(RenderData {
                transform: *transform,
                renderable: *renderable,
            });
        });
        planner.wait();

        while let Ok(cmd) = rx.try_recv() {
            window.draw(&duck_sprite);
        }

        window.display();
    }
}
