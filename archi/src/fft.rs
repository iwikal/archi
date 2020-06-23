use crate::context::Context;
use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{PipelineGate, TextureBinding},
    pixel::{Floating, RGBA32F},
    shader::{Program, Uniform},
    tess::{Mode, Tess},
    texture::{Dim2, GenMipmaps, Texture},
};
use luminance_derive::UniformInterface;
use luminance_gl::GL33;

const QUAD_VS_SRC: crate::shader::ShaderSource =
    crate::shader_source!("./shaders/quad.vert");

type TwiddleTexture = Texture<GL33, Dim2, RGBA32F>;

pub fn twiddle_indices(context: &mut Context, size: u32) -> TwiddleTexture {
    use luminance::texture::{MagFilter, MinFilter, Sampler};
    let mut sampler = Sampler::default();
    sampler.mag_filter = MagFilter::Nearest;
    sampler.min_filter = MinFilter::Nearest;

    let nf = size as f32;
    let bits = nf.log2() as u32;
    let width = bits;
    let height = size;
    let mut texture =
        Texture::new(context, [width, height], 0, sampler).unwrap();

    const TAU: f32 = std::f32::consts::PI * 2.0;

    let length = width * height;
    let mut pixels = Vec::with_capacity(length as usize);
    for y in 0..height {
        for x in 0..width {
            let span = u32::pow(2, x);

            let index = span * 2;

            let k = (y as f32 * nf / index as f32) % nf;
            let t = TAU * k / nf;

            let top_wing = y % index < span;

            let reverse = |i: u32| i.reverse_bits().rotate_left(bits);

            let (mut z, mut w) = if top_wing {
                (y, y + span)
            } else {
                (y - span, y)
            };

            if x == 0 {
                z = reverse(z);
                w = reverse(w);
            }

            pixels.push((t.cos(), t.sin(), z as f32, w as f32));
        }
    }

    texture.upload(GenMipmaps::No, &pixels).unwrap();

    texture
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

pub type FftTexture = Texture<GL33, Dim2, RGBA32F>;
pub type FftFramebuffer = Framebuffer<GL33, Dim2, RGBA32F, ()>;

pub fn fft_framebuffer(
    context: &mut Context,
    n: u32,
) -> anyhow::Result<FftFramebuffer> {
    use luminance::texture::{MagFilter, MinFilter, Sampler, Wrap};
    let sampler = Sampler {
        wrap_s: Wrap::Repeat,
        wrap_t: Wrap::Repeat,
        mag_filter: MagFilter::Linear,
        min_filter: MinFilter::LinearMipmapLinear,
        ..Default::default()
    };
    Ok(FftFramebuffer::new(context, [n, n], 0, sampler)?)
}

pub struct Fft {
    width: u32,
    twiddle_indices: TwiddleTexture,
    butterfly_shader: Program<GL33, (), (), ButterflyInterface>,
    inversion_shader: Program<GL33, (), (), InversionInterface>,
    pingpong_buffers: [FftFramebuffer; 2],
    tess: Tess<GL33, ()>,
}

impl Fft {
    pub fn new(context: &mut Context, width: u32) -> anyhow::Result<Self> {
        let twiddle_indices = twiddle_indices(context, width);

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
            fft_framebuffer(context, width)?,
            fft_framebuffer(context, width)?,
        ];

        let tess = context
            .new_tess()
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()?;

        Ok(Self {
            width,
            tess,
            twiddle_indices,
            butterfly_shader,
            inversion_shader,
            pingpong_buffers,
        })
    }

    pub fn render<'a>(
        &'a mut self,
        pipeline_gate: &mut PipelineGate<Context>,
        freq_texture: &mut FftTexture,
    ) -> &'a mut FftTexture {
        let Self {
            width,
            tess,
            twiddle_indices,
            butterfly_shader,
            inversion_shader,
            pingpong_buffers,
            ..
        } = self;

        let bits = (*width as f32).log2() as usize;

        let mut pingpong_buffers = {
            let [ping, pong] = pingpong_buffers;
            [ping, pong]
        };

        let mut initial_texture = Some(freq_texture);

        for &direction in &[0, 1] {
            for stage in 0..bits {
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
                        |pipeline, mut shader_gate| {
                            let bound_twiddle =
                                pipeline.bind_texture(twiddle_indices).unwrap();
                            let bound_input =
                                pipeline.bind_texture(texture).unwrap();
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
                                            tess_gate.render(&*tess);
                                        },
                                    );
                                },
                            );
                        },
                    )
                    .unwrap();

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
                    |pipeline, mut shader_gate| {
                        let bound_input =
                            pipeline.bind_texture(texture).unwrap();
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
                                    |mut tess_gate| {
                                        tess_gate.render(&*tess);
                                    },
                                );
                            },
                        );
                    },
                )
                .unwrap();

            out_buffer.color_slot()
        }
    }
}
