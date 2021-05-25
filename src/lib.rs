// TODO test how this affects fps + wasm size
// #![feature(allocator_api)]

#[macro_use]
mod utils;
use wasm_bindgen::prelude::*; // be interesting to separate luminance from wasm_bindgen

use luminance::context::GraphicsContext;
use luminance::render_state::RenderState;
// use luminance::shader::Program;
use luminance::tess::Mode;
use luminance_derive::{Semantics, Vertex};
use luminance_front::pipeline::PipelineState;
use luminance_web_sys::WebSysWebGL2Surface;

const CANVAS: &str = "canvas";
const VS_STR: &str = include_str!("shaders/vs.glsl");
const FS_STR: &str = include_str!("shaders/fs.glsl");

#[derive(Copy, Clone, Debug, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexRGB")]
    Color,
}

#[derive(Copy, Clone, Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    #[allow(dead_code)]
    position: VertexPosition,

    #[allow(dead_code)]
    #[vertex(normalized = "true")]
    color: VertexRGB,
}

const VERTICES: [Vertex; 3] = [
    Vertex::new(
        VertexPosition::new([-0.5, -0.5]),
        VertexRGB::new([255, 0, 0]),
    ),
    Vertex::new(
        VertexPosition::new([0.5, -0.5]),
        VertexRGB::new([0, 255, 0]),
    ),
    Vertex::new(VertexPosition::new([0., 0.5]), VertexRGB::new([0, 0, 255])),
];

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
#[allow(dead_code)]
pub struct Scene {}

#[wasm_bindgen]
impl Scene {
    pub fn new() -> Result<Scene, JsValue> {
        Ok(Scene {})
    }

    pub fn tick(&mut self, t: f32) -> Result<(), JsValue> {
        let mut surface = WebSysWebGL2Surface::new(CANVAS).expect("web-sys surface");
        let back_buffer = surface.back_buffer().unwrap();

        let color = [t.cos(), t.sin(), 0.5, 1.];

        let triangle = surface
            .new_tess()
            .set_vertices(&VERTICES[..])
            .set_mode(Mode::Triangle)
            .build()
            .unwrap(); // todo

        let mut program = surface
            .new_shader_program::<VertexSemantics, (), ()>()
            .from_strings(VS_STR, None, None, FS_STR)
            .unwrap()
            .ignore_warnings();

        let _ = surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color(color),
                |_, mut shd_gate| {
                    shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
                        rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                            tess_gate.render(&triangle)
                        })
                    })
                },
            )
            .assume();

        Ok(())
    }
}
