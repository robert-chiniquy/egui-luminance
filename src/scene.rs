use crate::object::Object;

use std::vec::Vec;

use luminance::context::GraphicsContext;
use luminance::render_state::RenderState;

// use luminance::shader::Program;

use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::pipeline::PipelineState;
use luminance_front::shader::Uniform;

use luminance_web_sys::WebSysWebGL2Surface;

// might be nice to consolidate to nalgebra
use cgmath::{perspective, EuclideanSpace, Matrix4, Point3, Rad, Vector3};
use nalgebra::Matrix;
use nalgebra::Point;
use ncollide3d::na::ArrayStorage;
use ncollide3d::na::U1;
use ncollide3d::na::U3;

const CANVAS: &str = "canvas"; // id of canvas in DOM
const VS_STR: &str = include_str!("shaders/vs.glsl");
const FS_STR: &str = include_str!("shaders/noise-fs.glsl");
// const FS_STR: &str = include_str!("shaders/debug-fs.glsl");

const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 10.;

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

impl From<&Point<f32, 3_usize>> for VertexPosition {
    fn from(p: &Point<f32, 3_usize>) -> Self {
        VertexPosition::new([p.x, p.y, p.z])
    }
}

impl From<&Matrix<f32, U3, U1, ArrayStorage<f32, 3, 1>>> for VertexNormal {
    fn from(m: &Matrix<f32, U3, U1, ArrayStorage<f32, 3, 1>>) -> Self {
        let s = m.as_slice();
        VertexNormal::new([s[0], s[1], s[2]])
    }
}

pub struct Scene {}

impl Scene {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, t: f32) {
        let mut surface = WebSysWebGL2Surface::new(CANVAS).expect("web-sys surface");
        let back_buffer = surface.back_buffer().unwrap();

        // square aspect ratio
        let projection = perspective(FOVY, 1., Z_NEAR, Z_FAR);
        let view = Matrix4::<f32>::look_at(
            Point3::new(t.sin(), t.cos(), -2.),
            Point3::origin(),
            Vector3::unit_y(),
        );

        let o = Object::default();
        let t = o.build_tess(&mut surface);

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
                        iface.set(&uni.projection, projection.into());
                        iface.set(&uni.view, view.into());
                        rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                            iface.set(&uni.model, o.model_matrix());
                            tess_gate.render(&t)
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
