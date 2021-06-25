use std::sync::Arc;
use std::vec::Vec;

use luminance::context::GraphicsContext;
use luminance::render_state::RenderState;

use luminance::blending::{Blending, Equation, Factor};
use luminance::pipeline::{PipelineState, TextureBinding};
use luminance::pixel::{NormUnsigned, SRGBA8UI};
use luminance::tess::Mode;
use luminance::texture::{Dim2, GenMipmaps, MinFilter, Sampler};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::shader::Uniform;
use luminance_front::tess::Tess;
use luminance_front::texture::Texture;
use luminance_front::Backend;
use luminance_web_sys::WebSysWebGL2Surface;

use egui::epaint::Texture as EguiTexture;
use egui::{CtxRef, RawInput};

const CANVAS: &str = "canvas";

const VS_STR: &str = include_str!("shaders/vertex_300es.glsl");
const FS_STR: &str = include_str!("shaders/fragment_300es.glsl");

pub type VertexIndex = u32;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Semantics)]
pub enum EguiVertexSemantics {
    #[sem(name = "a_pos", repr = "[f32; 2]", wrapper = "EguiVertexPosition")]
    Position,
    #[sem(name = "a_tc", repr = "[f32; 2]", wrapper = "EguiTextureCoords")]
    TextureCoords,
    #[sem(name = "a_srgba", repr = "[u8; 4]", wrapper = "EguiVertexColor")]
    Color,
}

#[derive(Clone, Copy, Debug, Vertex)]
#[vertex(sem = "EguiVertexSemantics")]
pub struct EguiVertex {
    position: EguiVertexPosition,
    tc: EguiTextureCoords,
    #[vertex(normalized = "true")]
    srgba: EguiVertexColor,
}

/*
From Luminance:
https://github.com/phaazon/luminance-rs/blob/master/luminance-webgl/src/webgl2/pixel.rs#L4
// WebGL format, internal sized-format and type.
pub(crate) fn webgl_pixel_format(pf: PixelFormat) -> Option<(u32, u32, u32)> {
  match (pf.format, pf.encoding) {

    (Format::SRGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::NormUnsigned) => {
      Some((
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::SRGB8_ALPHA8,
        WebGl2RenderingContext::UNSIGNED_BYTE,
      ))
    }
From Egui:
                let level = 0;
                let internal_format = Gl::SRGB8_ALPHA8;
                let border = 0;
                let src_format = Gl::RGBA;
                let src_type = Gl::UNSIGNED_BYTE;
                gl.pixel_storei(Gl::UNPACK_ALIGNMENT, 1);
                gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    Gl::TEXTURE_2D,
                    level,
                    internal_format as i32,
                    user_texture.size.0 as i32,
                    user_texture.size.1 as i32,
                    border,
                    src_format,
                    src_type,
                    Some(&pixels),
                )

- Introduce normalized texturing. That feature is encoded as pixel formats: any pixel format which
  symbol’s name starts with `Norm` is a _normalized pixel format_. Such formats state that the
  texels are encoded as integers but when fetched from a shader, they are turned into
  floating-point number by normalizing them. For instance, when fetching pixels from a texture
  encoded with `R8UI`, you get integers ranging in `[0; 255]` but when fetching pixels from a
  texture encoded with `NormR8UI`, even though texels are still stored as 8-bit unsigned integers,
  when fetched, you get floating-point numbers comprised in `[0; 1]`.
*/

#[derive(Debug, UniformInterface)]
struct EguiShaderInterface {
    #[uniform(unbound)]
    u_screen_size: Uniform<[f32; 2]>,
    #[uniform(unbound)]
    u_sampler: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

impl From<egui::Pos2> for EguiVertexPosition {
    fn from(p: egui::Pos2) -> Self {
        EguiVertexPosition::new([p.x, p.y])
    }
}

impl From<egui::Pos2> for EguiTextureCoords {
    fn from(p: egui::Pos2) -> Self {
        EguiTextureCoords::new([p.x, p.y])
    }
}

impl From<egui::Color32> for EguiVertexColor {
    fn from(c: egui::Color32) -> Self {
        EguiVertexColor::new(c.to_array())
    }
}

pub struct EguiLuminance {
    egui_ctx: CtxRef,
    egui_texture: Option<Arc<EguiTexture>>,
    egui_texture_size: [u32; 2],
    egui_texture_version: Option<u64>,
    canvas_size: [f32; 2],
    texels: Vec<u8>,
}

impl EguiLuminance {
    pub fn new() -> Self {
        Self {
            egui_ctx: CtxRef::default(),
            egui_texture: None,
            egui_texture_size: [0, 0],
            egui_texture_version: None,
            canvas_size: [0., 0.],
            texels: Vec::with_capacity(524288),
        }
    }

    fn write_egui_texture(&mut self, texture: &mut Texture<Dim2, SRGBA8UI>) {
        let egui_texture = match self.egui_texture.clone() {
            Some(et) => et,
            None => {
                panic!("No egui texture set!");
            }
        };

        if self.egui_texture_version == Some(egui_texture.version) {
            return;
        }

        self.texels = Vec::with_capacity(egui_texture.pixels.len());
        self.texels.clear();
        for srgba in egui_texture.srgba_pixels() {
            self.texels.push(srgba.r());
            self.texels.push(srgba.g());
            self.texels.push(srgba.b());
            self.texels.push(srgba.a());
        }

        let res = texture.upload_raw(GenMipmaps::No, &self.texels);
        match res {
            Ok(_) => {
                self.egui_texture_version = Some(egui_texture.version);
            }
            Err(_e) => {
                log!("{:?}", _e);
                panic!("texture upload error");
            }
        };
    }

    fn build_ui<C, F>(&mut self, surface: &mut C, builder: F) -> Tess<EguiVertex, VertexIndex>
    where
        C: GraphicsContext<Backend = Backend>,
        F: Fn(&CtxRef),
    {
        let i = RawInput {
            pixels_per_point: Some(2.0),
            ..RawInput::default()
        };

        self.egui_ctx.begin_frame(i);

        self.egui_texture = Some(self.egui_ctx.texture());
        self.egui_texture_size = [
            self.egui_ctx.texture().width as u32,
            self.egui_ctx.texture().height as u32,
        ];

        builder(&self.egui_ctx);

        let (_output, shapes) = self.egui_ctx.end_frame();
        // TODO handle output

        let clipped_meshes = self.egui_ctx.tessellate(shapes);

        // log!("clipped mesh length: {:?}", clipped_meshes.len());

        let indices: Vec<u32> = clipped_meshes[0].1.indices.iter().copied().collect();
        let vertices: Vec<EguiVertex> = clipped_meshes[0]
            .1
            .vertices
            .iter()
            .map(|v| EguiVertex {
                position: v.pos.into(),
                tc: v.uv.into(),
                srgba: v.color.into(),
            })
            .collect();

        // log!("{:?}", vertices);

        surface
            .new_tess()
            .set_vertices(vertices)
            .set_indices(indices)
            .set_mode(Mode::Triangle)
            .build()
            .unwrap()
    }

    pub fn render(&mut self, t: f32) {
        let mut surface = WebSysWebGL2Surface::new(CANVAS).expect("web-sys surface");
        self.canvas_size = [
            surface.canvas.width() as f32,
            surface.canvas.height() as f32,
        ];

        let u = self.build_ui(&mut surface, |ctx| {
            egui::SidePanel::left("❤", 200.).show(&ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("hello");
                });
                ui.separator();
                ui.label(format!("t: {}", t));
            });
        });

        let mut ui_tex: Texture<Dim2, SRGBA8UI> = Texture::new(
            &mut surface,
            self.egui_texture_size,
            0,
            Sampler {
                min_filter: MinFilter::Linear,
                ..Sampler::default()
            },
        )
        .expect("luminance texture creation");

        self.write_egui_texture(&mut ui_tex);

        // scissor region??
        let render_st = &RenderState::default()
            .set_blending_separate(
                Blending {
                    equation: Equation::Additive,
                    src: Factor::One,
                    dst: Factor::SrcAlphaComplement,
                },
                Blending {
                    equation: Equation::Additive,
                    src: Factor::One,
                    dst: Factor::SrcAlphaComplement,
                },
            )
            .set_depth_test(None);

        let pipeline_st = PipelineState::default()
            .enable_srgb(true)
            .set_clear_color([0.8, 0.8, 0.8, 1.]);

        let back_buffer = surface.back_buffer().unwrap();

        let building_program = surface
            .new_shader_program::<EguiVertexSemantics, (), EguiShaderInterface>()
            .from_strings(VS_STR, None, None, FS_STR);

        let built_program = match building_program {
            Ok(p) => p,
            Err(_e) => {
                log!("{:?}", _e);
                panic!("Can't build program");
            }
        };

        let mut program = built_program.ignore_warnings();

        let _ = surface
            .new_pipeline_gate()
            .pipeline(&back_buffer, &pipeline_st, |pipeline, mut shd_gate| {
                let bound_tex = pipeline.bind_texture(&mut ui_tex)?;

                shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
                    iface.set(&uni.u_screen_size, self.canvas_size);
                    iface.set(&uni.u_sampler, bound_tex.binding());
                    rdr_gate.render(&render_st, |mut tess_gate| tess_gate.render(&u))
                })
            })
            .assume();
    }
}

impl Default for EguiLuminance {
    fn default() -> Self {
        Self::new()
    }
}
