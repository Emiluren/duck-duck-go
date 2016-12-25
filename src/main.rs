#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate pegasus;
extern crate specs;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        pos: gfx::Global<[f32; 2]> = "u_Pos",
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const TRIANGLE: [Vertex; 3] = [
    Vertex { pos: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },
    Vertex { pos: [ 0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { pos: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
];

const CODE_VS: &'static[u8] = include_bytes!("shader/dot.vert");
const CODE_FS: &'static[u8] = include_bytes!("shader/dot.frag");

struct Drawable([f32; 2]);

impl specs::Component for Drawable {
    type Storage = specs::VecStorage<Drawable>;
}

struct MoveSystem;

impl specs::System<pegasus::Delta> for MoveSystem {
    fn run(&mut self, arg: specs::RunArg, t: pegasus::Delta) {
        use specs::Join;
        let mut vis = arg.fetch(|w| w.write::<Drawable>());
        let angle = t;
        let (c, s) = (angle.cos(), angle.sin());
        for &mut Drawable(ref mut p) in (&mut vis).iter() {
            *p = [p[0]*c - p[1]*s, p[0]*s + p[1]*c];
        }
    }
}

struct Init;

impl pegasus::Init for Init {
    type Shell = ();
    fn start(self, plan: &mut pegasus::Planner) -> () {
        plan.add_system(MoveSystem, "move", 20);
        let mut w = plan.mut_world();
        use std::f32::consts::PI;
        let num = 200;
        for i in 0..num {
            let t = i as f32 / (num as f32);
            let angle = t * 7.0 * PI;
            let pos = [t * angle.cos(), t * angle.sin()];
            w.create_now().with(Drawable(pos));
        }
    }
}

struct Painter<R: gfx::Resources> {
    slice: gfx::Slice<R>,
    pso: gfx::PipelineState<R, pipe::Meta>,
    data: pipe::Data<R>,
}

impl<R: gfx::Resources> pegasus::Painter<R> for Painter<R> {
    type Visual = Drawable;
    fn draw<'a, I, C>(&mut self, iter: I, enc: &mut gfx::Encoder<R, C>) where
        I: Iterator<Item = &'a Self::Visual>,
        C: gfx::CommandBuffer<R>
    {
        enc.clear(&self.data.out, [0.1, 0.2, 0.3, 1.0]);
        for &Drawable(pos) in iter {
            self.data.pos = pos.into();
            enc.draw(&self.slice, &self.pso, &self.data);
        }
    }
}

fn main() {
    use gfx::traits::FactoryExt;

    let builder = glutin::WindowBuilder::new()
        .with_title("Hello my friend :)".to_string())
        .with_dimensions(800, 600)
        .with_vsync();
    let (window, device, mut factory, main_color, _main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(
        &TRIANGLE, ()
    );

    let pso = factory.create_pipeline_simple(
        CODE_VS, CODE_FS, pipe::new()
    ).unwrap();

    let init = Init;
    let painter = Painter {
        slice: slice,
        pso: pso,
        data: pipe::Data {
            pos: [0.0, 0.0].into(),
            vbuf: vertex_buffer,
            out: main_color,
        }
    };

    pegasus::fly(window, device,
                 || factory.create_command_buffer(),
                 init, painter, ());
}
