use crate::context::Context;
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

fn quad_tess(context: &mut Context) -> anyhow::Result<Tess<()>> {
    let tess = context
        .new_tess()
        .set_mode(Mode::TriangleStrip)
        .set_vertex_nb(4)
        .build()?;

    Ok(tess)
}

pub type InitialDisplacementTexture = Texture<Dim2, RGBA32F>;
pub type DisplacementTexture = Texture<Dim2, RG32F>;

#[derive(UniformInterface)]
struct InitialDisplacementInterface {
    gauss_noise: Uniform<TextureBinding<Dim2, Floating>>,
    n: Uniform<i32>,
    scale: Uniform<i32>,
    amplitude: Uniform<f32>,
    intensity: Uniform<f32>, // wind speed
    direction: Uniform<[f32; 2]>,
    l: Uniform<f32>, // capillary supress factor
}

pub struct InitialDisplacement<const N: u32> {
    tess: Tess<()>,
    gauss_noise: Texture<Dim2, RGBA32F>,
    shader: Program<(), (), InitialDisplacementInterface>,
    pub framebuffer: Framebuffer<Dim2, RGBA32F, ()>,
    scale: i32,
    amplitude: f32,
    intensity: f32, // wind speed
    direction: glm::Vec2,
    l: f32, // capillary supress factor
}

impl<const N: u32> InitialDisplacement<N> {
    pub fn new(context: &mut Context) -> anyhow::Result<Self> {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0, Default::default())?;

        let shader = crate::shader::from_sources(
            context,
            None,
            super::QUAD_VS_SRC,
            None,
            crate::shader_source!("../shaders/h0k.frag"),
        )?;

        let gauss_noise = {
            use std::f32::consts::TAU;

            use luminance::texture::{MagFilter, MinFilter, Sampler};
            let sampler = Sampler {
                mag_filter: MagFilter::Nearest,
                min_filter: MinFilter::Nearest,
                ..Default::default()
            };

            let mut texture = Texture::new(context, size, 0, sampler)?;
            let length = N * N;
            let mut pixels = Vec::with_capacity(length as usize);
            let mut rng = rand::thread_rng();
            for _ in 0..length {
                use rand::Rng;
                let [a, b, c, d]: [f32; 4] = rng.gen();
                let a = (-2.0 * a.ln()).sqrt();
                let b = (-2.0 * b.ln()).sqrt();
                let c = TAU * c;
                let d = TAU * d;

                pixels.push((
                    a * c.cos(),
                    a * c.sin(),
                    b * d.cos(),
                    b * d.sin(),
                ));
            }

            texture.upload(GenMipmaps::No, &pixels)?;
            texture
        };

        let tess = quad_tess(context)?;

        Ok(Self {
            tess,
            gauss_noise,
            shader,
            framebuffer,
            scale: N as _,
            amplitude: 100.0,
            intensity: 90.0, // wind speed
            direction: glm::vec2(1.0, 1.0),
            l: 0.5,
        })
    }

    pub fn render(
        &mut self,
        pipeline_gate: &mut PipelineGate,
    ) -> anyhow::Result<&mut InitialDisplacementTexture> {
        let Self {
            framebuffer,
            gauss_noise,
            shader,
            tess,
            scale,
            amplitude,
            intensity,
            direction,
            l,
            ..
        } = self;

        pipeline_gate
            .pipeline(
                &*framebuffer,
                &Default::default(),
                |pipeline, mut shader_gate| -> anyhow::Result<()> {
                    let bound_noise = pipeline.bind_texture(gauss_noise)?;
                    shader_gate.shade(
                        shader,
                        |mut iface, uni, mut render_gate| {
                            iface.set(&uni.gauss_noise, bound_noise.binding());
                            iface.set(&uni.n, N as i32);
                            iface.set(&uni.scale, *scale);
                            iface.set(&uni.amplitude, *amplitude);
                            iface.set(&uni.intensity, *intensity);
                            iface.set(&uni.direction, (*direction).into());
                            iface.set(&uni.l, *l);
                            render_gate
                                .render(&Default::default(), |mut tess_gate| {
                                    tess_gate.render(&*tess)
                                })
                        },
                    )
                },
            )
            .into_result()?;

        Ok(framebuffer.color_slot())
    }

    pub fn into_texture(self) -> InitialDisplacementTexture {
        self.framebuffer.into_color_slot()
    }
}

#[derive(UniformInterface)]
struct DisplacementInterface {
    h0k_texture: Uniform<TextureBinding<Dim2, Floating>>,
    n: Uniform<i32>,
    time: Uniform<f32>,
}

pub struct Displacement<const N: u32> {
    tess: Tess<()>,
    shader: Program<(), (), DisplacementInterface>,
    pub framebuffer: Framebuffer<Dim2, (RG32F, RG32F, RG32F), ()>,
}

impl<const N: u32> Displacement<N> {
    pub fn new(context: &mut Context) -> anyhow::Result<Self> {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0, Default::default())?;
        let shader = crate::shader::from_sources(
            context,
            None,
            super::QUAD_VS_SRC,
            None,
            crate::shader_source!("../shaders/hkt.frag"),
        )?;

        let tess = quad_tess(context)?;

        Ok(Self {
            tess,
            shader,
            framebuffer,
        })
    }

    pub fn render(
        &mut self,
        pipeline_gate: &mut PipelineGate,
        time: f32,
        h0k_texture: &mut InitialDisplacementTexture,
    ) -> anyhow::Result<[&mut DisplacementTexture; 3]> {
        let Self {
            framebuffer,
            shader,
            tess,
            ..
        } = self;

        pipeline_gate
            .pipeline(
                &framebuffer,
                &Default::default(),
                |pipeline, mut shader_gate| -> anyhow::Result<()> {
                    let bound_h0k = pipeline.bind_texture(h0k_texture)?;
                    shader_gate.shade(
                        shader,
                        |mut iface, uni, mut render_gate| {
                            iface.set(&uni.h0k_texture, bound_h0k.binding());
                            iface.set(&uni.n, N as i32);
                            iface.set(&uni.time, time);
                            render_gate
                                .render(&Default::default(), |mut tess_gate| {
                                    tess_gate.render(&*tess)
                                })
                        },
                    )
                },
            )
            .into_result()?;

        let (x, y, z) = framebuffer.color_slot();
        Ok([x, y, z])
    }
}
