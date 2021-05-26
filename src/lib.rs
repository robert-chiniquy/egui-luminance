// TODO test how this affects fps + wasm size
// #![feature(allocator_api)]

#[macro_use]
mod utils;

use ncollide3d::na::ArrayStorage;
use ncollide3d::na::U1;
use ncollide3d::na::U3;
use wasm_bindgen::prelude::*; // be interesting to separate luminance from wasm_bindgen

// use std::convert::TryFrom;
use std::vec::Vec;

use luminance::context::GraphicsContext;
use luminance::render_state::RenderState;
// use luminance::tess::Tess;
// use luminance::shader::Program;
use luminance::tess::Mode;
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::pipeline::PipelineState;
use luminance_front::shader::Uniform;
use luminance_web_sys::WebSysWebGL2Surface;

use cgmath::{perspective, EuclideanSpace, Matrix4, Point3, Rad, Vector3};
use nalgebra::Matrix;
use nalgebra::Point;

#[allow(unused_imports)]
use ncollide3d::{
    procedural::IndexBuffer, procedural::TriMesh, shape::Ball, shape::Cuboid,
    transformation::ToTriMesh,
};

const CANVAS: &str = "canvas";
const VS_STR: &str = include_str!("shaders/vs.glsl");
const FS_STR: &str = include_str!("shaders/fs.glsl");

const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 10.;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "normal", repr = "[f32; 3]", wrapper = "VertexNormal")]
    Normal,
}

#[derive(Clone, Copy, Debug, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct Vertex {
    position: VertexPosition,
    normal: VertexNormal,
}

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
    #[uniform(unbound)]
    projection: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    view: Uniform<[[f32; 4]; 4]>,
}

type VertexIndex = u32;

struct Model {
    vertices: Vec<Vertex>,
    indices: Vec<VertexIndex>,
    // todo add model translation matrix? pool mesh data?
}

impl From<TriMesh<f32>> for Model {
    fn from(mut t: TriMesh<f32>) -> Self {
        t.unify_index_buffer();

        let indices: Vec<VertexIndex> = t.flat_indices();

        let n = t.normals;
        let vertices = t
            .coords
            .iter()
            .zip(n.unwrap().iter())
            .into_iter()
            .map(|(p, m)| Vertex::new(p.into(), m.into()))
            .collect();

        Model { vertices, indices }
    }
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

// not a matrix3
// impl From<&Matrix3<f32>> for VertexNormal {
//     fn from(m: &Matrix3<f32>) -> Self {
//         let s = m.as_slice();
//         VertexNormal::new([s[0], s[1], s[2]])
//     }
// }

// const VERTICES: [Vertex; 3] = [
//     Vertex::new(VertexPosition::new([-0.5, -0.5, 1.])),
//     Vertex::new(VertexPosition::new([0.5, -0.5, -1.])),
//     Vertex::new(VertexPosition::new([0., 0.5, 0.])),
// ];

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

        // todo cleanup width and height etc
        let projection = perspective(FOVY, 960. / 540., Z_NEAR, Z_FAR);
        let view = Matrix4::<f32>::look_at(
            Point3::new(t.cos() * 2., t.sin() * 2., 2.),
            Point3::origin(),
            Vector3::unit_y(),
        );

        let ball = Ball::new(1.0f32);
        let trimesh = ball.to_trimesh((10, 10));

        // let cuboid = Cuboid::new(Vector3::new(1.0f32, 1.0, 1.0));
        // let mut trimesh = cuboid.to_trimesh(());

        let m: Model = trimesh.into();

        let triangle = surface
            .new_tess()
            .set_vertices(m.vertices)
            .set_indices(m.indices)
            .set_mode(Mode::Triangle)
            .build()
            .unwrap(); // todo error sense

        let mut program = surface
            .new_shader_program::<VertexSemantics, (), ShaderInterface>()
            .from_strings(VS_STR, None, None, FS_STR)
            .unwrap()
            .ignore_warnings();

        let _ = surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color(color),
                |_, mut shd_gate| {
                    shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
                        iface.set(&uni.projection, projection.into());
                        iface.set(&uni.view, view.into());
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
