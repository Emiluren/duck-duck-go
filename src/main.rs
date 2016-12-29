#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate pegasus;
extern crate specs;
extern crate image;
extern crate glob;
extern crate nalgebra as na;

use std::io::Cursor;
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use std::sync::Arc;
use glob::glob;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_UV",
    }

    pipeline pipe {
        pos: gfx::Global<[f32; 2]> = "u_Pos",
        aspect: gfx::Global<f32> = "u_Aspect",
        vbuf: gfx::VertexBuffer<Vertex> = (),
        current_texture: gfx::TextureSampler<[f32; 4]> = "tex",
        out: gfx::BlendTarget<ColorFormat> = ("Target0",
                                              gfx::state::ColorMask::all(),
                                              gfx::preset::blend::ALPHA),
    }
}

const QUAD: [Vertex; 6] = [
    Vertex { pos: [-0.5, -0.5], uv: [0.0, 1.0] },
    Vertex { pos: [ 0.5, -0.5], uv: [1.0, 1.0] },
    Vertex { pos: [-0.5,  0.5], uv: [0.0, 0.0] },

    Vertex { pos: [-0.5,  0.5], uv: [0.0, 0.0] },
    Vertex { pos: [ 0.5, -0.5], uv: [1.0, 1.0] },
    Vertex { pos: [ 0.5,  0.5], uv: [1.0, 0.0] },
];

const CODE_VS: &'static[u8] = include_bytes!("shader/sprite.vert");
const CODE_FS: &'static[u8] = include_bytes!("shader/sprite.frag");

//const DUCK_CRAB_SCALE: f32 = 0.1;

struct Sprite {
    pos: [f32; 2],
    gpu_data_index: usize,
}


impl specs::Component for Sprite {
    type Storage = specs::VecStorage<Sprite>;
}

struct Transform {
    pos: [f32; 2],
}

impl specs::Component for Transform {
    type Storage = specs::VecStorage<Transform>;
}

struct Spinner;

impl specs::Component for Spinner {
    type Storage = specs::HashMapStorage<Spinner>;
}

struct SpinSystem;

impl specs::System<pegasus::Delta> for SpinSystem {
    fn run(&mut self, arg: specs::RunArg, t: pegasus::Delta) {
        use specs::Join;
        let (mut space, spinners) = arg.fetch(|w| {
            (w.write::<Transform>(), w.read::<Spinner>())
        });
        let angle = t;
        let (c, s) = (angle.cos(), angle.sin());
        for (ref mut ent, _) in (&mut space, &spinners).iter() {
            let p = ent.pos;
            ent.pos = [p[0]*c - p[1]*s, p[0]*s + p[1]*c];
        }
    }
}

struct PreDrawSystem;

impl specs::System<pegasus::Delta> for PreDrawSystem {
    fn run(&mut self, arg: specs::RunArg, _: pegasus::Delta) {
        use specs::Join;
        let (mut draw, space) = arg.fetch(|w| {
            (w.write::<Sprite>(), w.read::<Transform>())
        });

        for (d, s) in (&mut draw, &space).iter() {
            d.pos = s.pos;
        }
    }
}

fn create_sprite_entity(world: &mut specs::World, start_pos: [f32; 2], texture_index: usize) -> specs::Entity {
    world.create_now()
        .with(Transform { pos: start_pos })
        .with(Sprite { pos: start_pos, gpu_data_index: texture_index })
        .build()
}

struct Init {
    texture_indices: Arc<HashMap<String, usize>>,
}

impl pegasus::Init for Init {
    type Shell = ();
    fn start(self, plan: &mut pegasus::Planner) -> () {
        plan.add_system(SpinSystem, "move", 20);
        plan.add_system(PreDrawSystem, "pre_draw", pegasus::DRAW_PRIORITY + 5);

        let crab = {
            let mut w = plan.mut_world();

            w.register::<Transform>();
            w.register::<Spinner>();

            create_sprite_entity(w, [0.5, 0.0], *self.texture_indices.get("assets/duck1.png").unwrap());
            create_sprite_entity(w, [-0.5, 0.0], *self.texture_indices.get("assets/crab1.png").unwrap())
        };

        plan.run_custom(move |arg| {
            let mut spinners = arg.fetch(|w| w.write::<Spinner>());
            spinners.insert(crab, Spinner);
        });
    }
}

struct Painter<R: gfx::Resources> {
    slice: gfx::Slice<R>,
    pso: gfx::PipelineState<R, pipe::Meta>,
    pipelines: Vec<pipe::Data<R>>,
    out_color: gfx::handle::RenderTargetView<R, ColorFormat>,
    //texture_indices: Arc<HashMap<String, usize>>,
}

impl <R: gfx::Resources> Painter<R> {
}

impl<R: gfx::Resources> pegasus::Painter<R> for Painter<R> {
    type Visual = Sprite;
    fn draw<'a, I, C>(&mut self, iter: I, enc: &mut gfx::Encoder<R, C>) where
        I: Iterator<Item = &'a Self::Visual>,
        C: gfx::CommandBuffer<R>
    {
        enc.clear(&self.out_color, [0.1, 0.2, 0.3, 1.0]);

        for ref mut sprite in iter {
            let ref mut pipe = self.pipelines[sprite.gpu_data_index];
            pipe.pos = sprite.pos.into();
            enc.draw(&self.slice, &self.pso, pipe);
        }
    }
}

fn load_texture<R, F>(factory: &mut F, filename: &str) ->
    gfx::handle::ShaderResourceView<R, [f32; 4]> where R: gfx::Resources, F: gfx::Factory<R>
{
    use gfx::format::Rgba8;
    use gfx::tex as t;
    let mut file = File::open(filename).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single);
    let (_, view) = factory.create_texture_const_u8::<Rgba8>(kind, &[&img]).unwrap();
    view
}

fn main() {
    use gfx::traits::FactoryExt;

    let builder = glutin::WindowBuilder::new()
        .with_title("Hello my friend :)".to_string())
        .with_dimensions(800, 600)
        .with_vsync();
    let (window, device, mut factory, main_color, _main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    let (window_width, window_height) = window.get_inner_size_points().unwrap();
    let aspect = (window_width as f32)/(window_height as f32);

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(
        &QUAD, ()
    );

    let pso = factory.create_pipeline_simple(
        CODE_VS, CODE_FS, pipe::new()
    ).unwrap();

    let sampler = factory.create_sampler_linear();

    let create_pipe_data = |texture| pipe::Data {
        pos: [0.0, 0.0].into(),
        aspect: aspect,
        vbuf: vertex_buffer.clone(),
        current_texture: (texture, sampler.clone()),
        out: main_color.clone(),
    };

    let mut texture_map = HashMap::new();
    let mut pipelines = Vec::new();
    for texture_file in glob("assets/*.png").unwrap() {
        let texture_file = texture_file.unwrap();
        let filename = texture_file.to_str().unwrap();
        println!("loading {}", filename);

        let current_index = pipelines.len();
        let texture = load_texture(&mut factory, filename);
        texture_map.insert(filename.to_string(), current_index);
        pipelines.push(create_pipe_data(texture));
    }

    let texture_indices = Arc::new(texture_map);

    let init = Init { texture_indices: texture_indices.clone() };
    let painter = Painter {
        slice: slice,
        pso: pso,
        pipelines: pipelines,
        out_color: main_color.clone(),
        //texture_indices: texture_indices.clone(),
    };

    pegasus::fly(window, device,
                 || factory.create_command_buffer(),
                 init, painter, ());
}
