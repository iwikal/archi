use luminance::{
    context::GraphicsContext,
    linear::M44,
    pipeline::{BoundTexture, Builder, Pipeline, ShadingGate},
    pixel::Floating,
    shader::program::{Program, Uniform},
    tess::Tess,
    texture::{Dim2, Flat},
};
use luminance_derive::UniformInterface;

#[derive(UniformInterface)]
pub struct OceanShaderInterface {
    #[uniform(unbound)]
    heightmap: Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    view_projection: Uniform<M44>,
    offset: Uniform<[f32; 2]>,
}

type OceanShader = Program<(), (), OceanShaderInterface>;

use crate::fft::{Fft, FftFramebuffer, H0k, Hkt};
pub struct Ocean {
    h0k: H0k,
    hkt: Hkt,
    fft: Fft,
    heightmap_buffer: FftFramebuffer,
    shader: OceanShader,
    tess: Tess,
}

impl Ocean {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let h0k = H0k::new(context);
        {
            let mut builder = context.pipeline_builder();
            h0k.render(&mut builder);
        }
        let hkt = Hkt::new(context);
        let fft = Fft::new(context);
        let heightmap_buffer = crate::fft::fft_framebuffer(context);
        let shader = crate::shader::from_sources(
            Some((
                crate::shader_source!("./shaders/ocean.tesc"),
                crate::shader_source!("./shaders/ocean.tese"),
            )),
            crate::shader_source!("./shaders/ocean.vert"),
            None, // Some(crate::shader_source!("./shaders/ocean.geom")),
            crate::shader_source!("./shaders/ocean.frag"),
        );

        let tess = crate::grid::square_patch_grid(context, 0x100);

        Self {
            h0k,
            hkt,
            fft,
            heightmap_buffer,
            shader,
            tess,
        }
    }

    pub fn simulate(
        &mut self,
        builder: &mut Builder<impl GraphicsContext>,
        time: f32,
    ) -> OceanFrame {
        let Self {
            h0k,
            hkt,
            fft,
            heightmap_buffer,
            ..
        } = self;
        hkt.render(builder, time, h0k.framebuffer.color_slot());
        fft.render(builder, hkt.framebuffer.color_slot(), heightmap_buffer);
        OceanFrame(self)
    }
}

pub struct OceanFrame<'a>(&'a Ocean);

impl<'a> OceanFrame<'a> {
    pub fn render(
        &self,
        pipeline: &Pipeline,
        shader_gate: &mut ShadingGate<impl GraphicsContext>,
        view_projection: glm::Mat4,
    ) {
        let Self(Ocean {
            heightmap_buffer,
            shader,
            tess,
            ..
        }) = self;

        let heightmap = pipeline.bind_texture(heightmap_buffer.color_slot());
        shader_gate.shade(shader, |iface, mut render_gate| {
            iface.view_projection.update(view_projection.into());
            iface.heightmap.update(&heightmap);
            render_gate.render(&Default::default(), |mut tess_gate| {
                for x in -1..1 {
                    for y in -1..1 {
                        iface.offset.update([x as f32, y as f32]);
                        tess_gate.render(tess);
                    }
                }
            });
        })
    }
}
