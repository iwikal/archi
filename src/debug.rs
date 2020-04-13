#![allow(dead_code)]
use luminance::{
    context::GraphicsContext,
    pipeline::BoundTexture,
    pixel::RGB32F,
    shader::{Program, Uniform},
    tess::{Mode, Tess, TessBuilder},
    texture::{Dim2},
};
use luminance_gl::GL33;
use luminance_derive::UniformInterface;

type M44 = [[f32; 4]; 4];

#[derive(UniformInterface)]
pub struct DebugShaderInterface {
    input_texture:
        Uniform<&'static BoundTexture<'static, GL33, Dim2, RGB32F>>,
    view_projection: Uniform<M44>,
    model: Uniform<M44>,
}

impl DebugShaderInterface {
    pub fn set_texture(&self, t: &BoundTexture<'_, GL33, Dim2, RGB32F>) {
        self.input_texture.update(t);
    }

    pub fn set_model(&self, m: impl Into<M44>) {
        self.model.update(m.into());
    }

    pub fn set_view_projection(&self, vp: impl Into<M44>) {
        self.view_projection.update(vp.into());
    }
}

pub struct Debugger {
    pub shader: Program<GL33, (), (), DebugShaderInterface>,
    pub tess: Tess<GL33>,
}

impl Debugger {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let shader = crate::shader::from_strings(
            None,
            include_str!("./shaders/framebuffer-debug.vert"),
            include_str!("./shaders/framebuffer-debug.frag"),
        );

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        Self { shader, tess }
    }
}
