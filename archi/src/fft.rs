use crate::context::Context;
use luminance::texture::Sampler;
use luminance_derive::UniformInterface;
use luminance_front::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{PipelineGate, TextureBinding},
    pixel::{Floating, RG32F, RGBA32F},
    shader::{Program, Uniform},
    tess::{Mode, Tess},
    texture::{Dim2, GenMipmaps, Texture},
};
use std::f32::consts::TAU;

const QUAD_VS_SRC: crate::shader::ShaderSource =
    crate::shader_source!("./shaders/quad.vert");

type TwiddleTexture = Texture<Dim2, RGBA32F>;

fn twiddle_indices(
    context: &mut Context,
    size: u32,
) -> anyhow::Result<TwiddleTexture> {
    use luminance::texture::{MagFilter, MinFilter};
    let sampler = Sampler {
        mag_filter: MagFilter::Nearest,
        min_filter: MinFilter::Nearest,
        ..Default::default()
    };

    let nf = size as f32;
    let stages = nf.log2() as u32;
    let width = stages;
    let height = size;
    let mut texture = context.new_texture([width, height], 0, sampler)?;

    let length = width * height;
    let mut pixels = Vec::with_capacity(length as usize);
    for y in 0..height {
        for x in 0..width {
            let span = u32::pow(2, x);

            let index = span * 2;

            let k = (y as f32 * nf / index as f32) % nf;
            let t = TAU * k / nf;

            let top_wing = y % index < span;

            let reverse = |i: u32| i.reverse_bits().rotate_left(stages);

            let (mut twiddle_u, mut twiddle_v) = if top_wing {
                (y, y + span)
            } else {
                (y - span, y)
            };

            // reverse twiddle indices on the first iteration
            if x == 0 {
                twiddle_u = reverse(twiddle_u);
                twiddle_v = reverse(twiddle_v);
            }

            let omega_real = t.cos();
            let omega_imag = t.sin();

            pixels.push((
                omega_real,
                omega_imag,
                twiddle_u as f32,
                twiddle_v as f32,
            ));
        }
    }

    texture.upload(GenMipmaps::No, &pixels)?;

    Ok(texture)
}

#[derive(UniformInterface)]
struct ButterflyInterface {
    twiddle_indices: Uniform<TextureBinding<Dim2, Floating>>,
    input_texture: Uniform<TextureBinding<Dim2, Floating>>,
    stage: Uniform<i32>,
    direction: Uniform<i32>,
}

#[derive(UniformInterface)]
struct InversionInterface {
    input_texture: Uniform<TextureBinding<Dim2, Floating>>,
    n: Uniform<u32>,
}

pub type FftTexture = Texture<Dim2, RG32F>;
pub type FftFramebuffer = Framebuffer<Dim2, RG32F, ()>;

pub struct Fft {
    width: u32,
    twiddle_indices: TwiddleTexture,
    butterfly_shader: Program<(), (), ButterflyInterface>,
    inversion_shader: Program<(), (), InversionInterface>,
    pingpong_buffers: [FftFramebuffer; 2],
    tess: Tess<()>,
}

impl Fft {
    pub fn new(context: &mut Context, width: u32) -> anyhow::Result<Self> {
        use luminance_front::texture::{MagFilter, MinFilter, Wrap};

        let sampler = Sampler {
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
            mag_filter: MagFilter::Linear,
            min_filter: MinFilter::LinearMipmapLinear,
            ..Default::default()
        };

        Self::with_sampler(context, width, sampler)
    }

    pub fn with_sampler(
        context: &mut Context,
        width: u32,
        sampler: Sampler,
    ) -> anyhow::Result<Self> {
        let twiddle_indices = twiddle_indices(context, width)?;

        let butterfly_shader = crate::shader::from_sources(
            context,
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/butterfly.frag"),
        )?;

        let inversion_shader = crate::shader::from_sources(
            context,
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/inversion.frag"),
        )?;

        let pingpong_buffers = [
            context.new_framebuffer([width, width], 0, sampler)?,
            context.new_framebuffer([width, width], 0, sampler)?,
        ];

        let tess = context
            .new_tess()
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()?;

        Ok(Self {
            width,
            twiddle_indices,
            butterfly_shader,
            inversion_shader,
            pingpong_buffers,
            tess,
        })
    }

    pub fn render<'a>(
        &'a mut self,
        pipeline_gate: &mut PipelineGate,
        freq_texture: &mut FftTexture,
    ) -> anyhow::Result<&'a mut FftTexture> {
        let Self {
            width,
            tess,
            twiddle_indices,
            butterfly_shader,
            inversion_shader,
            pingpong_buffers,
            ..
        } = self;

        let stages = 31 - u32::leading_zeros(*width);

        let mut pingpong_buffers = {
            let [ping, pong] = pingpong_buffers;
            [ping, pong]
        };

        let mut initial_texture = Some(freq_texture);

        for &direction in &[0, 1] {
            for stage in 0..stages {
                let [in_buffer, out_buffer] = pingpong_buffers;

                let texture = match initial_texture {
                    Some(t) => {
                        initial_texture = None;
                        t
                    }
                    None => in_buffer.color_slot(),
                };

                pipeline_gate
                    .pipeline(
                        out_buffer,
                        &Default::default(),
                        |pipeline, mut shader_gate| -> anyhow::Result<()> {
                            let bound_twiddle =
                                pipeline.bind_texture(twiddle_indices)?;
                            let bound_input = pipeline.bind_texture(texture)?;
                            shader_gate.shade(
                                butterfly_shader,
                                |mut iface, uni, mut render_gate| {
                                    iface.set(
                                        &uni.twiddle_indices,
                                        bound_twiddle.binding(),
                                    );
                                    iface.set(
                                        &uni.input_texture,
                                        bound_input.binding(),
                                    );
                                    iface.set(&uni.stage, stage as i32);
                                    iface.set(&uni.direction, direction);
                                    render_gate.render(
                                        &Default::default(),
                                        |mut tess_gate| {
                                            tess_gate.render(&*tess)
                                        },
                                    )
                                },
                            )
                        },
                    )
                    .into_result()?;

                pingpong_buffers = [out_buffer, in_buffer];
            }
        }
        {
            let [in_buffer, out_buffer] = pingpong_buffers;

            let texture = in_buffer.color_slot();

            pipeline_gate
                .pipeline(
                    out_buffer,
                    &Default::default(),
                    |pipeline, mut shader_gate| -> anyhow::Result<()> {
                        let bound_input = pipeline.bind_texture(texture)?;
                        shader_gate.shade(
                            inversion_shader,
                            |mut iface, uni, mut render_gate| {
                                iface.set(
                                    &uni.input_texture,
                                    bound_input.binding(),
                                );
                                iface.set(&uni.n, *width);
                                render_gate.render(
                                    &Default::default(),
                                    |mut tess_gate| tess_gate.render(&*tess),
                                )
                            },
                        )
                    },
                )
                .into_result()?;

            Ok(out_buffer.color_slot())
        }
    }

    pub fn into_target_texture(self) -> FftTexture {
        let [_, buf] = self.pingpong_buffers;
        buf.into_color_slot()
    }
}
