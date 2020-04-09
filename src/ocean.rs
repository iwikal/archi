use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    linear::M44,
    pipeline::{BoundTexture, Builder, Pipeline, ShadingGate},
    pixel::{Floating, RGB32F, RGBA32F},
    shader::program::{Program, Uniform},
    tess::Tess,
    tess::{Mode, TessBuilder},
    texture::{Dim2, Flat, GenMipmaps, Texture},
};
use luminance_derive::UniformInterface;

const QUAD_VS_SRC: crate::shader::ShaderSource =
    crate::shader_source!("./shaders/quad.vert");

#[derive(UniformInterface)]
struct H0kInterface {
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    n: Uniform<i32>,
    scale: Uniform<i32>,
    amplitude: Uniform<f32>,
    intensity: Uniform<f32>, // wind speed
    direction: Uniform<[f32; 2]>,
    l: Uniform<f32>, // capillary supress factor
}

type H0kTexture = Texture<Flat, Dim2, RGBA32F>;

struct H0k {
    tess: Tess,
    input_texture: Texture<Flat, Dim2, RGBA32F>,
    shader: Program<(), (), H0kInterface>,
    framebuffer: Framebuffer<Flat, Dim2, RGBA32F, ()>,
    scale: i32,
    amplitude: f32,
    intensity: f32, // wind speed
    direction: glm::Vec2,
    l: f32, // capillary supress factor
}

const N: u32 = 0x400;

impl H0k {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0, Default::default())
                .expect("framebuffer creation");

        let shader = crate::shader::from_sources(
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/h0k.frag"),
        );

        use luminance::texture::{MagFilter, MinFilter, Sampler};
        let mut sampler = Sampler::default();
        sampler.mag_filter = MagFilter::Nearest;
        sampler.min_filter = MinFilter::Nearest;

        let input_texture = Texture::new(context, size, 0, sampler).unwrap();
        {
            let length = N * N;
            let mut pixels = Vec::with_capacity(length as usize);
            let mut rng = rand::thread_rng();
            for _ in 0..length {
                use rand::Rng;
                pixels.push(rng.gen());
            }

            input_texture.upload(GenMipmaps::No, &pixels).unwrap();
        }

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        Self {
            tess,
            input_texture,
            shader,
            framebuffer,
            scale: 1000,
            amplitude: 4.0,
            intensity: 40.0, // wind speed
            direction: glm::vec2(1.0, 1.0),
            l: 0.5, // capillary supress factor
        }
    }

    fn render(
        &self,
        builder: &mut Builder<impl GraphicsContext>,
    ) -> &H0kTexture {
        let Self {
            framebuffer,
            input_texture,
            shader,
            tess,
            ..
        } = self;
        builder.pipeline(
            framebuffer,
            &Default::default(),
            |pipeline, mut shader_gate| {
                let bound_noise = pipeline.bind_texture(input_texture);
                shader_gate.shade(shader, |iface, mut render_gate| {
                    iface.input_texture.update(&bound_noise);
                    iface.n.update(N as i32);
                    iface.scale.update(self.scale);
                    iface.amplitude.update(self.amplitude);
                    iface.intensity.update(self.intensity);
                    iface.direction.update(self.direction.into());
                    iface.l.update(self.l);
                    render_gate.render(&Default::default(), |mut tess_gate| {
                        tess_gate.render(tess);
                    });
                });
            },
        );
        framebuffer.color_slot()
    }
}

#[derive(UniformInterface)]
struct HktInterface {
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    n: Uniform<i32>,
    time: Uniform<f32>,
}

type HktTexture = Texture<Flat, Dim2, RGBA32F>;

struct Hkt {
    tess: Tess,
    shader: Program<(), (), HktInterface>,
    framebuffer: Framebuffer<Flat, Dim2, RGBA32F, ()>,
}

impl Hkt {
    fn new(context: &mut impl GraphicsContext) -> Self {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0, Default::default())
                .expect("framebuffer creation");
        let shader = crate::shader::from_sources(
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/hkt.frag"),
        );

        use luminance::texture::{MagFilter, MinFilter, Sampler};
        let mut sampler = Sampler::default();
        sampler.mag_filter = MagFilter::Nearest;
        sampler.min_filter = MinFilter::Nearest;

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        Self {
            tess,
            shader,
            framebuffer,
        }
    }

    fn render(
        &self,
        builder: &mut Builder<impl GraphicsContext>,
        time: f32,
        input_texture: &H0kTexture,
    ) -> &HktTexture {
        let Self {
            framebuffer,
            shader,
            tess,
            ..
        } = self;
        builder.pipeline(
            framebuffer,
            &Default::default(),
            |pipeline, mut shader_gate| {
                let bound_noise = pipeline.bind_texture(input_texture);
                shader_gate.shade(shader, |iface, mut render_gate| {
                    iface.input_texture.update(&bound_noise);
                    iface.n.update(N as i32);
                    iface.time.update(time);
                    render_gate.render(&Default::default(), |mut tess_gate| {
                        tess_gate.render(tess);
                    });
                });
            },
        );
        framebuffer.color_slot()
    }
}

#[derive(UniformInterface)]
pub struct OceanShaderInterface {
    #[uniform(unbound)]
    heightmap: Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    #[uniform(unbound)]
    normalmap: Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    view_projection: Uniform<M44>,
    offset: Uniform<[f32; 2]>,
}

type OceanShader = Program<(), (), OceanShaderInterface>;

#[derive(UniformInterface)]
pub struct NormalShaderInterface {
    #[uniform(unbound)]
    heightmap: Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
}

type NormalShader = Program<(), (), NormalShaderInterface>;
type NormalFramebuffer = Framebuffer<Flat, Dim2, RGB32F, ()>;
type NormalTexture = Texture<Flat, Dim2, RGB32F>;

struct NormalGenerator {
    tess: Tess,
    shader: NormalShader,
    framebuffer: NormalFramebuffer,
}

impl NormalGenerator {
    fn new(context: &mut impl GraphicsContext) -> Self {
        let shader = crate::shader::from_sources(
            None,
            QUAD_VS_SRC,
            None,
            crate::shader_source!("./shaders/ocean-normal.frag"),
        );

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        use luminance::texture::{MagFilter, MinFilter, Sampler, Wrap};
        let sampler = Sampler {
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
            mag_filter: MagFilter::Linear,
            min_filter: MinFilter::LinearMipmapLinear,
            ..Default::default()
        };
        let framebuffer = Framebuffer::new(context, [N, N], 0, sampler)
            .expect("framebuffer creation");

        Self {
            shader,
            tess,
            framebuffer,
        }
    }

    fn render(
        &self,
        builder: &mut Builder<impl GraphicsContext>,
        heightmap: &H0kTexture,
    ) -> &NormalTexture {
        let Self {
            framebuffer,
            shader,
            tess,
            ..
        } = self;
        builder.pipeline(
            framebuffer,
            &Default::default(),
            |pipeline, mut shader_gate| {
                shader_gate.shade(shader, |iface, mut render_gate| {
                    iface.heightmap.update(&pipeline.bind_texture(heightmap));
                    render_gate.render(&Default::default(), |mut tess_gate| {
                        tess_gate.render(tess);
                    });
                });
            },
        );
        framebuffer.color_slot()
    }
}

use crate::fft::{Fft, FftTexture};
pub struct Ocean {
    h0k: H0k,
    hkt: Hkt,
    fft: Fft,
    normal_generator: NormalGenerator,
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
        let fft = Fft::new(context, N);
        let shader = crate::shader::from_sources(
            Some((
                crate::shader_source!("./shaders/ocean.tesc"),
                crate::shader_source!("./shaders/ocean.tese"),
            )),
            crate::shader_source!("./shaders/ocean.vert"),
            None, // Some(crate::shader_source!("./shaders/ocean.geom")),
            crate::shader_source!("./shaders/ocean.frag"),
        );

        let normal_generator = NormalGenerator::new(context);

        let tess = crate::grid::square_patch_grid(context, 0x100);

        Self {
            h0k,
            hkt,
            fft,
            shader,
            normal_generator,
            tess,
        }
    }

    pub fn simulate(
        &mut self,
        builder: &mut Builder<impl GraphicsContext>,
        time: f32,
    ) -> OceanFrame {
        let heightmap = {
            self.hkt
                .render(builder, time, self.h0k.framebuffer.color_slot());
            self.fft.render(builder, self.hkt.framebuffer.color_slot())
        };
        let normalmap = self.normal_generator.render(builder, heightmap);

        OceanFrame {
            shader: &self.shader,
            tess: &self.tess,
            heightmap,
            normalmap,
        }
    }
}

pub struct OceanFrame<'a> {
    shader: &'a OceanShader,
    tess: &'a Tess,
    heightmap: &'a FftTexture,
    normalmap: &'a NormalTexture,
}

impl<'a> OceanFrame<'a> {
    pub fn render(
        &self,
        pipeline: &Pipeline,
        shader_gate: &mut ShadingGate<impl GraphicsContext>,
        view_projection: glm::Mat4,
    ) {
        let &Self {
            shader,
            tess,
            heightmap,
            normalmap,
        } = self;

        let heightmap = pipeline.bind_texture(heightmap);
        let normalmap = pipeline.bind_texture(normalmap);
        shader_gate.shade(shader, |iface, mut render_gate| {
            iface.view_projection.update(view_projection.into());
            iface.heightmap.update(&heightmap);
            iface.normalmap.update(&normalmap);
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
