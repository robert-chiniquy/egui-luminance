use crate::scene::{Vertex, VertexColor, VertexIndex, VertexPosition};

use luminance::tess::Mode;
use luminance_front::context::GraphicsContext;
use luminance_front::tess::Tess;
use luminance_front::Backend;
use nalgebra::{Matrix4, Vector3};
use ncollide3d::{procedural::TriMesh, shape::Ball, transformation::ToTriMesh};
use std::f32::consts::PI;

pub struct Object {
    vertices: Vec<Vertex>,
    indices: Vec<VertexIndex>,
    translation: Vector3<f32>,
    rotation: Matrix4<f32>,
    scale: Matrix4<f32>,
}

impl Default for Object {
    fn default() -> Self {
        Self::new(
            VertexColor::new([51, 51, 255]),
            Vector3::new(0.0, 0.0, 0.0),
            Matrix4::from_scaled_axis(Vector3::x() * PI),
            Matrix4::new_scaling(1.0),
        )
    }
}

impl Object {
    pub fn new(
        color: VertexColor,
        translation: Vector3<f32>,
        rotation: Matrix4<f32>,
        scale: Matrix4<f32>,
    ) -> Self {
        let ball = Ball::new(1.0f32);
        let trimesh = ball.to_trimesh((16, 16));

        // let cuboid = Cuboid::new(Vector3::new(1.0f32, 1.0, 1.0));
        // let mut trimesh = cuboid.to_trimesh(());

        let mut o = Object {
            vertices: vec![],
            indices: vec![],
            translation,
            rotation,
            scale,
        };

        o.load_trimesh(|_| color, trimesh);

        o
    }

    pub fn translate(&mut self, v: Vector3<f32>) {
        self.translation += v;
    }

    pub fn rotate(&mut self, axisangle: Vector3<f32>) {
        self.rotation *= Matrix4::from_scaled_axis(axisangle);
    }

    pub fn model_matrix(&self) -> [[f32; 4]; 4] {
        let model_matrix = self.rotation * self.scale.append_translation(&self.translation);
        model_matrix.into()
    }

    // pass a closure to produce a vertex color attribute
    fn load_trimesh<F: Fn(VertexPosition) -> VertexColor>(
        &mut self,
        f: F,
        mut trimesh: TriMesh<f32>,
    ) {
        trimesh.unify_index_buffer();

        let indices: Vec<VertexIndex> = trimesh.flat_indices();

        let n = trimesh.normals;
        let vertices = trimesh
            .coords
            .iter()
            .zip(n.unwrap().iter())
            .into_iter()
            .map(|(p, m)| -> Vertex { Vertex::new(p.into(), m.into(), f(p.into())) })
            .collect();

        self.indices = indices;
        self.vertices = vertices;
    }

    pub fn build_tess<C>(&self, surface: &mut C) -> Tess<Vertex, VertexIndex>
    where
        C: GraphicsContext<Backend = Backend>,
    {
        surface
            .new_tess()
            .set_vertices(self.vertices.clone())
            .set_indices(self.indices.clone())
            .set_mode(Mode::TriangleStrip)
            .build()
            .unwrap() // todo error handling
    }
}
