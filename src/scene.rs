use std::sync::Arc;
use std::vec::Vec;

use luminance::context::GraphicsContext;
use luminance::render_state::RenderState;

use luminance::blending::{Blending, Equation, Factor};
use luminance::pipeline::{PipelineState, TextureBinding};
use luminance::pixel::{NormUnsigned, SRGBA8UI};
use luminance::tess::Mode;
use luminance::texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler};
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

pub struct Scene {
    egui_ctx: CtxRef,
    egui_texture: Option<Arc<EguiTexture>>,
    egui_texture_size: [u32; 2],
    egui_texture_version: Option<u64>,
    canvas_size: [f32; 2],
}

impl Scene {
    pub fn new() -> Self {
        Self {
            egui_ctx: CtxRef::default(),
            egui_texture: None,
            egui_texture_size: [0, 0],
            egui_texture_version: None,
            canvas_size: [0., 0.],
        }
    }

    //    fn egui_texture_needs_update

    fn write_egui_texture(&mut self, texture: &mut Texture<Dim2, SRGBA8UI>) {
        let egui_texture = match self.egui_texture.clone() {
            Some(et) => et,
            None => {
                //log!("No egui texture set!");
                panic!("No egui texture set!");
            }
        };

        if self.egui_texture_version == Some(egui_texture.version) {
            return;
        }

        // log!(
        //     "Loading egui texture, pixels: {:?}",
        //     egui_texture.pixels.len()
        // );

        let mut texels: Vec<(u8, u8, u8, u8)> = Vec::with_capacity(egui_texture.pixels.len());

        for srgba in egui_texture.srgba_pixels() {
            texels.push((srgba.r(), srgba.g(), srgba.b(), srgba.a()));
        }

        //log!("{:?}", texels);

        let res = texture.upload(GenMipmaps::No, &texels);
        match res {
            Ok(_) => {
                self.egui_texture_version = Some(egui_texture.version);
            }
            Err(_e) => {
                //log!("{:?}", e);
                panic!("texture upload error");
            }
        };
    }

    fn build_ui<C, F>(&mut self, surface: &mut C, builder: F) -> Tess<EguiVertex, VertexIndex>
    where
        C: GraphicsContext<Backend = Backend>,
        F: Fn(&CtxRef),
    {
        let i = RawInput::default();
        // todo handle input
        self.egui_ctx.begin_frame(i);

        // todo factor out egui texture stuff
        self.egui_texture = Some(self.egui_ctx.texture());
        // todo optimize for minimum copies / maximum sense
        self.egui_texture_size = [
            self.egui_ctx.texture().width as u32,
            self.egui_ctx.texture().height as u32,
        ];

        builder(&self.egui_ctx);

        let (_output, shapes) = self.egui_ctx.end_frame();
        // TODO handle output

        let clipped_meshes = self.egui_ctx.tessellate(shapes);

        // For now just grab the first mesh, maybe it is one mesh per panel?
        // need to remain alert for when it needs to account for multiple meshes
        // log!("egui mesh count: {:?}", clipped_meshes.len());

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

        //log!("egui vertices: {:?}", &vertices);
        //log!("egui indices: {:?}", indices);
        //vertices.iter().for_each(|v| log!("{:?} ", v.srgba));
        //vertices.iter().for_each(|v| log!("{:?} ", v.tc));

        let u: Tess<EguiVertex, VertexIndex> = surface
            .new_tess()
            .set_vertices(vertices)
            .set_indices(indices)
            .set_mode(Mode::TriangleStrip)
            .build()
            .unwrap();

        u
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

        // todo reduce loading calls in hot loop
        // egui_web has:
        //  let internal_format = Gl::SRGB8_ALPHA8;
        //  let src_type = Gl::UNSIGNED_BYTE;
        /*
        pub(crate) fn webgl_pixel_format(pf: PixelFormat) -> Option<(u32, u32, u32)> {
        match (pf.format, pf.encoding) {

            (Format::SRGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::NormUnsigned) => {
            Some((
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::SRGB8_ALPHA8,
                WebGl2RenderingContext::UNSIGNED_BYTE,
            ))
            }
        */
        let mut ui_tex: Texture<Dim2, SRGBA8UI> = Texture::new(
            &mut surface,
            self.egui_texture_size,
            0,
            Sampler {
                // wrap_r: Wrap::ClampToEdge,
                // wrap_s: Wrap::ClampToEdge,
                // wrap_t: Wrap::ClampToEdge,
                min_filter: MinFilter::Nearest,
                mag_filter: MagFilter::Nearest,
                // depth_comparison: None,
                ..Sampler::default()
            },
        )
        .expect("luminance texture creation");
        self.write_egui_texture(&mut ui_tex);

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
        // todo nicely print any warnings
        // log!("{:?}", built_program.warnings);

        let mut program = built_program.ignore_warnings();

        // egui uses premultiplied alpha, so make sure your blending function is (ONE, ONE_MINUS_SRC_ALPHA).
        // Factor::SrcAlphaComplement => WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        // Backface culling is disabled by default
        // egui_web uses a scissor region
        let render_st = &RenderState::default().set_blending(Blending {
            equation: Equation::Additive,
            src: Factor::One,
            dst: Factor::SrcAlphaComplement,
        });

        let _ = surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color([0.8, 0.8, 0.8, 1.]),
                |pipeline, mut shd_gate| {
                    let bound_tex = pipeline.bind_texture(&mut ui_tex)?;

                    shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
                        iface.set(&uni.u_screen_size, self.canvas_size);
                        iface.set(&uni.u_sampler, bound_tex.binding());
                        rdr_gate.render(&render_st, |mut tess_gate| tess_gate.render(&u))
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
