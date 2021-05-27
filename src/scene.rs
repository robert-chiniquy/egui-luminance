use std::vec::Vec;

use luminance::context::GraphicsContext;
use luminance::render_state::RenderState;

// use luminance::shader::Program;

use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::pipeline::PipelineState;
use luminance_front::shader::Uniform;

use luminance_web_sys::WebSysWebGL2Surface;

use egui::{CtxRef, RawInput};

const CANVAS: &str = "canvas"; // id of canvas in DOM
const VS_STR: &str = include_str!("shaders/fragment_100es.glsl");
const FS_STR: &str = include_str!("shaders/vertex_100es.glsl");

pub type VertexIndex = u32;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "normal", repr = "[f32; 3]", wrapper = "VertexNormal")]
    Normal,
    #[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexColor")]
    Color,
}

#[derive(Clone, Copy, Debug, Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    position: VertexPosition,
    normal: VertexNormal,
    #[vertex(normalized = "true")]
    rgb: VertexColor,
}

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
    #[uniform(unbound)]
    projection: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    view: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    model: Uniform<[[f32; 4]; 4]>,
}

// impl From<&Point<f32, 3_usize>> for VertexPosition {
//     fn from(p: &Point<f32, 3_usize>) -> Self {
//         VertexPosition::new([p.x, p.y, p.z])
//     }
// }

// impl From<&Matrix<f32, U3, U1, ArrayStorage<f32, 3, 1>>> for VertexNormal {
//     fn from(m: &Matrix<f32, U3, U1, ArrayStorage<f32, 3, 1>>) -> Self {
//         let s = m.as_slice();
//         VertexNormal::new([s[0], s[1], s[2]])
//     }
// }

pub struct Scene {
    egui_ctx: CtxRef,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            egui_ctx: CtxRef::default(),
        }
    }

    pub fn ui(&mut self, t: f32) {
        let i = RawInput::default();
        self.egui_ctx.begin_frame(i);
        egui::SidePanel::left("❤", 200.).show(&self.egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("hello");
            });
            ui.separator();
            ui.label(format!("t: {}", t));
        });
        let (_output, shapes) = self.egui_ctx.end_frame();
        // TODO handle output
        let clipped_meshes = self.egui_ctx.tessellate(shapes);
    }

    pub fn render(&self, t: f32) {
        let mut surface = WebSysWebGL2Surface::new(CANVAS).expect("web-sys surface");
        let back_buffer = surface.back_buffer().unwrap();

        let mut program = surface
            .new_shader_program::<VertexSemantics, (), ShaderInterface>()
            .from_strings(VS_STR, None, None, FS_STR)
            .unwrap()
            .ignore_warnings();

        let _ = surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color([0.9, 0.9, 0.9, 1.]),
                |_, mut shd_gate| {
                    shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
                        // iface.set(&uni.projection, projection.into());
                        // iface.set(&uni.view, view.into());
                        rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                            // tess_gate.render(&t)
                            Ok(())
                        })
                    })
                },
            )
            .assume();
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
