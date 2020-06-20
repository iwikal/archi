#![allow(dead_code)]
use crate::context::Context;
use luminance::{
    context::GraphicsContext,
    pipeline::{Pipeline, TextureBinding},
    pixel::{Floating, RGBA32F},
    shader::{Program, Uniform},
    shading_gate::ShadingGate,
    tess::{Mode, Tess},
    texture::{Dim2, Texture},
};
use luminance_derive::UniformInterface;
use luminance_gl::GL33;

pub mod glerr;

type M44 = [[f32; 4]; 4];

#[derive(UniformInterface)]
pub struct DebugShaderInterface {
    #[uniform(unbound)]
    input_texture: Uniform<TextureBinding<Dim2, Floating>>,
    view_projection: Uniform<M44>,
    model: Uniform<M44>,
}

pub struct Debugger {
    shader: Program<GL33, (), (), DebugShaderInterface>,
    tess: Tess<GL33, ()>,
}

impl Debugger {
    pub fn new(context: &mut Context) -> Self {
        let shader = crate::shader::from_sources(
            context,
            None,
            crate::shader_source!("./shaders/framebuffer-debug.vert"),
            None,
            crate::shader_source!("./shaders/framebuffer-debug.frag"),
        );

        let tess = context
            .new_tess()
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        Self { shader, tess }
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline<GL33>,
        shader_gate: &mut ShadingGate<Context>,
        view_projection: impl Into<M44>,
        model: impl Into<M44>,
        texture: Option<&mut Texture<GL33, Dim2, RGBA32F>>,
    ) {
        let Self { shader, tess } = self;

        shader_gate.shade(shader, |mut iface, uni, mut render_gate| {
            iface.set(&uni.view_projection, view_projection.into());
            iface.set(&uni.model, model.into());

            if let Some(texture) = texture {
                let bound_texture = pipeline.bind_texture(texture).unwrap();
                iface.set(&uni.input_texture, bound_texture.binding());
            }

            render_gate.render(&Default::default(), |mut tess_gate| {
                tess_gate.render(&*tess);
            });
        })
    }
}
