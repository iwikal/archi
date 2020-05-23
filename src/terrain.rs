use crate::context::Context;
use image::GenericImageView;
use luminance::{
    pipeline::{Pipeline, TextureBinding},
    pixel::{Floating, R32F},
    shader::{
        Program, Uniform, UniformBuilder, UniformInterface, UniformWarning,
    },
    shading_gate::ShadingGate,
    tess::Tess,
    texture::{Dim2, Texture},
};
use luminance_gl::GL33;

type M44 = [[f32; 4]; 4];

pub struct TerrainShaderInterface {
    heightmap: Uniform<TextureBinding<Dim2, Floating>>,
    view_projection: Uniform<M44>,
}

impl UniformInterface<GL33, ()> for TerrainShaderInterface {
    fn uniform_interface<'a>(
        builder: &mut UniformBuilder<'a, GL33>,
        _env: &mut (),
    ) -> Result<Self, UniformWarning> {
        Ok(TerrainShaderInterface {
            heightmap: builder.ask("heightmap")?,
            view_projection: builder.ask("view_projection")?,
        })
    }
}

type TerrainShader = Program<GL33, (), (), TerrainShaderInterface>;

pub struct Terrain {
    heightmap: Texture<GL33, Dim2, R32F>,
    shader: TerrainShader,
    tess: Tess<GL33, (), u32>,
}

impl Terrain {
    pub fn new(
        context: &mut Context,
        heightmap: impl AsRef<std::path::Path>,
    ) -> Self {
        let heightmap = {
            use luminance::texture::{
                GenMipmaps, MagFilter, MinFilter, Sampler,
            };
            let mut sampler = Sampler::default();
            sampler.mag_filter = MagFilter::Nearest;
            sampler.min_filter = MinFilter::Nearest;
            let heightmap = heightmap.as_ref();
            let mut image = image::open(heightmap).unwrap_or_else(|e| {
                panic!("could not load texture {:?}: {}", heightmap, e);
            });
            let min = u32::min(image.width(), image.height());
            let image = image.crop(0, 0, min, min);
            let image = image.resize_exact(
                0x100,
                0x100,
                image::imageops::FilterType::Triangle,
            );
            let size = [image.width(), image.height()];
            let mut texture = Texture::new(context, size, 0, sampler).unwrap();

            let pixels: Vec<f32> = image
                .raw_pixels()
                .into_iter()
                .map(|p| {
                    let p = f32::from(p) / 255.0;
                    p * 20.0 - 3.0
                })
                .collect();

            texture.upload(GenMipmaps::Yes, &pixels).unwrap();
            texture
        };
        let tess = crate::grid::strip_grid(context, 0x100);
        let shader = crate::shader::from_strings(
            context,
            None,
            include_str!("./shaders/terrain.vert"),
            include_str!("./shaders/terrain.frag"),
        );
        Self {
            heightmap,
            shader,
            tess,
        }
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline<GL33>,
        shader_gate: &mut ShadingGate<Context>,
        view_projection: impl Into<M44>,
    ) {
        let Self {
            heightmap,
            shader,
            tess,
        } = self;

        let bound_heightmap = pipeline.bind_texture(heightmap).unwrap();

        shader_gate.shade(shader, |mut iface, uni, mut render_gate| {
            iface.set(&uni.view_projection, view_projection.into());
            iface.set(&uni.heightmap, bound_heightmap.binding());
            render_gate.render(&Default::default(), |mut tess_gate| {
                tess_gate.render(&*tess);
            });
        })
    }
}
