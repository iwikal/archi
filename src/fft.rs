use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{BoundTexture, Builder},
    pixel::{Floating, RGBA32F},
    shader::program::{Program, Uniform},
    tess::{Mode, Tess, TessBuilder},
    texture::{Dim2, Flat, GenMipmaps, Texture},
};
use luminance_derive::UniformInterface;

const QUAD_VS_SRC: crate::shader::ShaderSource =
    crate::shader_source!("./shaders/quad.vert");

type TwiddleTexture = Texture<Flat, Dim2, RGBA32F>;

pub fn twiddle_indices(
    context: &mut impl GraphicsContext,
    size: u32,
) -> TwiddleTexture {
    use luminance::texture::{MagFilter, MinFilter, Sampler};
    let mut sampler = Sampler::default();
    sampler.mag_filter = MagFilter::Nearest;
    sampler.min_filter = MinFilter::Nearest;

    let nf = size as f32;
    let bits = nf.log2() as u32;
    let width = bits;
    let height = size;
    let texture = Texture::new(context, [width, height], 0, sampler).unwrap();
    {
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
    }

    texture
}

#[derive(UniformInterface)]
struct ButterflyInterface {
    twiddle_indices:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    stage: Uniform<i32>,
    direction: Uniform<i32>,
}

#[derive(UniformInterface)]
struct InversionInterface {
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    n: Uniform<u32>,
}

pub type FftTexture = Texture<Flat, Dim2, RGBA32F>;
pub type FftFramebuffer = Framebuffer<Flat, Dim2, RGBA32F, ()>;

pub fn fft_framebuffer(
    context: &mut impl GraphicsContext,
    n: u32,
) -> FftFramebuffer {
    use luminance::texture::{MagFilter, MinFilter, Sampler, Wrap};
    let sampler = Sampler {
        wrap_s: Wrap::Repeat,
        wrap_t: Wrap::Repeat,
        mag_filter: MagFilter::Linear,
        min_filter: MinFilter::LinearMipmapLinear,
        ..Default::default()
    };
    FftFramebuffer::new(context, [n, n], 0, sampler)
        .expect("fft framebuffer creation")
}

pub struct Fft {
    width: u32,
    twiddle_indices: TwiddleTexture,
    butterfly_shader: Program<(), (), ButterflyInterface>,
    inversion_shader: Program<(), (), InversionInterface>,
    pingpong_buffers: [FftFramebuffer; 2],
    tess: Tess,
}

impl Fft {
    pub fn new(context: &mut impl GraphicsContext, width: u32) -> Self {
        let twiddle_indices = twiddle_indices(context, width);

        let butterfly_shader = crate::shader::from_sources(
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/butterfly.frag"),
        );

        let inversion_shader = crate::shader::from_sources(
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/inversion.frag"),
        );

        let pingpong_buffers = [
            fft_framebuffer(context, width),
            fft_framebuffer(context, width),
        ];

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        Self {
            width,
            tess,
            twiddle_indices,
            butterfly_shader,
            inversion_shader,
            pingpong_buffers,
        }
    }

    pub fn render<'a>(
        &'a self,
        builder: &mut Builder<impl GraphicsContext>,
        input_texture: &FftTexture,
    ) -> &'a FftTexture {
        let Self {
            width,
            tess,
            twiddle_indices,
            butterfly_shader,
            inversion_shader,
            ..
        } = self;

        let bits = (*width as f32).log2() as usize;
        let mut pingpong = bits % 2;
        let mut first_round = true;

        for &direction in &[0, 1] {
            for stage in 0..bits {
                let input = if first_round {
                    first_round = false;
                    input_texture
                } else {
                    self.pingpong_buffers[pingpong].color_slot()
                };
                let output = &self.pingpong_buffers[1 - pingpong];

                builder.pipeline(
                    output,
                    &Default::default(),
                    |pipeline, mut shader_gate| {
                        let bound_twiddle =
                            pipeline.bind_texture(twiddle_indices);
                        let bound_input = pipeline.bind_texture(input);
                        shader_gate.shade(
                            butterfly_shader,
                            |iface, mut render_gate| {
                                iface.twiddle_indices.update(&bound_twiddle);
                                iface.input_texture.update(&bound_input);
                                iface.stage.update(stage as i32);
                                iface.direction.update(direction);
                                render_gate.render(
                                    &Default::default(),
                                    |mut tess_gate| {
                                        tess_gate.render(tess);
                                    },
                                );
                            },
                        );
                    },
                );
                pingpong = 1 - pingpong;
            }
        }
        {
            let input = self.pingpong_buffers[pingpong].color_slot();
            let output = &self.pingpong_buffers[1 - pingpong];
            builder.pipeline(
                output,
                &Default::default(),
                |pipeline, mut shader_gate| {
                    let bound_input = pipeline.bind_texture(input);
                    shader_gate.shade(
                        inversion_shader,
                        |iface, mut render_gate| {
                            iface.input_texture.update(&bound_input);
                            iface.n.update(*width);
                            render_gate.render(
                                &Default::default(),
                                |mut tess_gate| {
                                    tess_gate.render(tess);
                                },
                            );
                        },
                    );
                },
            );
            output.color_slot()
        }
    }
}
