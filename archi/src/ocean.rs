use crate::context::Context;
use luminance_derive::UniformInterface;
use luminance_front::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{Pipeline, PipelineGate, TextureBinding},
    pixel::{Floating, RGB32F, RGBA32F},
    shader::{Program, Uniform},
    shading_gate::ShadingGate,
    tess::{Mode, Tess},
    texture::{Dim2, GenMipmaps, Texture},
};

const QUAD_VS_SRC: crate::shader::ShaderSource =
    crate::shader_source!("./shaders/quad.vert");

fn quad_tess(context: &mut Context) -> anyhow::Result<Tess<()>> {
    let tess = context
        .new_tess()
        .set_mode(Mode::TriangleStrip)
        .set_vertex_nb(4)
        .build()?;

    Ok(tess)
}

#[derive(UniformInterface)]
struct H0kInterface {
    input_texture: Uniform<TextureBinding<Dim2, Floating>>,
    n: Uniform<i32>,
    scale: Uniform<i32>,
    amplitude: Uniform<f32>,
    intensity: Uniform<f32>, // wind speed
    direction: Uniform<[f32; 2]>,
    l: Uniform<f32>, // capillary supress factor
}

type H0kTexture = Texture<Dim2, RGBA32F>;

struct H0k {
    tess: Tess<()>,
    noise: Texture<Dim2, RGBA32F>,
    shader: Program<(), (), H0kInterface>,
    framebuffer: Framebuffer<Dim2, RGBA32F, ()>,
    scale: i32,
    amplitude: f32,
    intensity: f32, // wind speed
    direction: glm::Vec2,
    l: f32, // capillary supress factor
}

const N: u32 = 0x200;

impl H0k {
    pub fn new(context: &mut Context) -> anyhow::Result<Self> {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0, Default::default())?;

        let shader = crate::shader::from_sources(
            context,
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/h0k.frag"),
        )?;

        let noise = {
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
                pixels.push(rng.gen());
            }

            texture.upload(GenMipmaps::No, &pixels)?;
            texture
        };

        let tess = quad_tess(context)?;

        Ok(Self {
            tess,
            noise,
            shader,
            framebuffer,
            scale: N as _,
            amplitude: 1.0 / 2.0,
            intensity: 40.0, // wind speed
            direction: glm::vec2(1.0, 1.0),
            l: 0.5, // capillary supress factor
        })
    }

    fn render(
        &mut self,
        pipeline_gate: &mut PipelineGate,
    ) -> anyhow::Result<&mut H0kTexture> {
        let Self {
            framebuffer,
            noise,
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
                    let bound_noise = pipeline.bind_texture(noise)?;
                    shader_gate.shade(
                        shader,
                        |mut iface, uni, mut render_gate| {
                            iface
                                .set(&uni.input_texture, bound_noise.binding());
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

    fn into_texture(self) -> H0kTexture {
        self.framebuffer.into_color_slot()
    }
}

#[derive(UniformInterface)]
struct HktInterface {
    input_texture: Uniform<TextureBinding<Dim2, Floating>>,
    n: Uniform<i32>,
    time: Uniform<f32>,
}

type HktTexture = Texture<Dim2, RGBA32F>;

struct Hkt {
    tess: Tess<()>,
    shader: Program<(), (), HktInterface>,
    framebuffer: Framebuffer<Dim2, RGBA32F, ()>,
}

impl Hkt {
    fn new(context: &mut Context) -> anyhow::Result<Self> {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0, Default::default())?;
        let shader = crate::shader::from_sources(
            context,
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/hkt.frag"),
        )?;

        let tess = quad_tess(context)?;

        Ok(Self {
            tess,
            shader,
            framebuffer,
        })
    }

    fn render(
        &mut self,
        pipline_gate: &mut PipelineGate,
        time: f32,
        h0k_texture: &mut H0kTexture,
    ) -> anyhow::Result<&HktTexture> {
        let Self {
            framebuffer,
            shader,
            tess,
            ..
        } = self;

        pipline_gate
            .pipeline(
                &framebuffer,
                &Default::default(),
                |pipeline, mut shader_gate| -> anyhow::Result<()> {
                    let bound_h0k = pipeline.bind_texture(h0k_texture)?;
                    shader_gate.shade(
                        shader,
                        |mut iface, uni, mut render_gate| {
                            iface.set(&uni.input_texture, bound_h0k.binding());
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

        Ok(framebuffer.color_slot())
    }
}

#[derive(UniformInterface)]
pub struct OceanShaderInterface {
    heightmap: Uniform<TextureBinding<Dim2, Floating>>,
    view_projection: Uniform<[[f32; 4]; 4]>,
    camera_offset: Uniform<[f32; 2]>,

    sky_texture: Uniform<TextureBinding<Dim2, Floating>>,
    camera_pos: Uniform<[f32; 3]>,
    exposure: Uniform<f32>,
}

type OceanShader = Program<(), (), OceanShaderInterface>;

use crate::fft::{Fft, FftTexture};
pub struct Ocean {
    h0k_texture: H0kTexture,
    hkt: Hkt,
    fft: Fft,
    shader: OceanShader,
    tess: Tess<(), u32>,
}

impl Ocean {
    pub fn new(context: &mut Context) -> anyhow::Result<Self> {
        let mut h0k = H0k::new(context)?;
        h0k.render(&mut context.new_pipeline_gate())?;

        let hkt = Hkt::new(context)?;
        let fft = Fft::new(context, N)?;
        let shader = crate::shader::from_sources(
            context,
            Some((
                crate::shader_source!("./shaders/ocean.tesc"),
                crate::shader_source!("./shaders/ocean.tese"),
            )),
            crate::shader_source!("./shaders/ocean.vert"),
            None, // Some(crate::shader_source!("./shaders/ocean.geom")),
            crate::shader_source!("./shaders/ocean.frag"),
        )?;

        let tess = crate::grid::square_patch_grid(context, 0x100)?;

        let h0k_texture = h0k.into_texture();

        Ok(Self {
            h0k_texture,
            hkt,
            fft,
            shader,
            tess,
        })
    }

    pub fn simulate(
        &mut self,
        pipeline_gate: &mut PipelineGate,
        time: f32,
    ) -> anyhow::Result<OceanFrame> {
        let heightmap = {
            self.hkt
                .render(pipeline_gate, time, &mut self.h0k_texture)?;
            self.fft
                .render(pipeline_gate, self.hkt.framebuffer.color_slot())?
        };

        Ok(OceanFrame {
            shader: &mut self.shader,
            tess: &mut self.tess,
            heightmap,
        })
    }
}

pub struct OceanFrame<'a> {
    shader: &'a mut OceanShader,
    tess: &'a mut Tess<(), u32>,
    pub heightmap: &'a mut FftTexture,
}

impl<'a> OceanFrame<'a> {
    pub fn render(
        &mut self,
        pipeline: &Pipeline,
        shader_gate: &mut ShadingGate,
        view_projection: glm::Mat4,
        camera_pos: glm::Vec3,
        sky_texture: Option<&mut Texture<Dim2, RGB32F>>,
        exposure: f32,
    ) -> anyhow::Result<()> {
        let Self {
            shader,
            tess,
            heightmap,
        } = self;

        let heightmap = pipeline.bind_texture(heightmap)?;

        shader_gate.shade(shader, |mut iface, uni, mut render_gate| {
            iface.set(&uni.view_projection, view_projection.into());
            iface.set(&uni.heightmap, heightmap.binding());

            iface.set(&uni.camera_pos, camera_pos.into());
            if let Some(texture) = sky_texture {
                let texture = pipeline.bind_texture(texture)?;
                iface.set(&uni.sky_texture, texture.binding());
            }
            iface.set(&uni.exposure, exposure);

            render_gate.render(&Default::default(), |mut tess_gate| {
                iface.set(&uni.camera_offset, [camera_pos.x, camera_pos.z]);
                tess_gate.render(&**tess)
            })
        })
    }
}
