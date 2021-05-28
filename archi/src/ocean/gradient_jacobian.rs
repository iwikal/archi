use crate::context::Context;
use luminance_derive::UniformInterface;
use luminance_front::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pixel::{RGB32F, Floating},
    pipeline::{TextureBinding, PipelineGate},
    tess::{Tess, Mode},
    texture::{Sampler, Wrap},
    shader::{Program, Uniform},
    texture::{Dim2, Texture},
};

use super::displacement;

#[derive(UniformInterface)]
struct DispGradJacInterface {
    xmap: Uniform<TextureBinding<Dim2, Floating>>,
    ymap: Uniform<TextureBinding<Dim2, Floating>>,
    zmap: Uniform<TextureBinding<Dim2, Floating>>,

    choppiness: Uniform<f32>,
}

/// A stage in the ocean pipeline, which bakes a displacement map and a combined map of the
/// gradient and jacobian determinant of the displacement.
pub struct DispGradJac<const N: u32> {
    shader: Program<(), (RGB32F, RGB32F), DispGradJacInterface>,
    framebuffer: Framebuffer<Dim2, (RGB32F, RGB32F), ()>,
    tess: Tess<()>,
}

impl<const N: u32> DispGradJac<N> {
    pub fn new(context: &mut Context) -> anyhow::Result<Self> {
        let shader = crate::shader::from_sources(
            context,
            None,
            crate::shader_source!("../shaders/quad.vert"),
            None,
            crate::shader_source!("../shaders/bake_water_maps.frag"),
        )?;

        let sampler = Sampler {
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
            ..Default::default()
        };

        let tess = context
            .new_tess()
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()?;

        Ok(Self {
            shader,
            framebuffer: context.new_framebuffer([N, N], 0, sampler)?,
            tess,
        })
    }

    pub fn render(
        &mut self,
        pipeline_gate: &mut PipelineGate,
        xmap: &mut displacement::DisplacementTexture,
        ymap: &mut displacement::DisplacementTexture,
        zmap: &mut displacement::DisplacementTexture,
    ) -> anyhow::Result<&mut (Texture<Dim2, RGB32F>, Texture<Dim2, RGB32F>)> {
        let Self {
            shader,
            framebuffer,
            tess,
        } = self;

        pipeline_gate.pipeline(
            &framebuffer,
            &Default::default(),
            |pipeline, mut shader_gate| -> anyhow::Result<()> {
                let xmap = pipeline.bind_texture(xmap)?;
                let ymap = pipeline.bind_texture(ymap)?;
                let zmap = pipeline.bind_texture(zmap)?;
                shader_gate.shade(shader, |mut iface, uni, mut render_gate| {
                    iface.set(&uni.xmap, xmap.binding());
                    iface.set(&uni.ymap, ymap.binding());
                    iface.set(&uni.zmap, zmap.binding());
                    render_gate.render(&Default::default(), |mut tess_gate| {
                        tess_gate.render(&*tess)
                    })
                })
            },
        ).into_result()?;

        Ok(framebuffer.color_slot())
    }
}
