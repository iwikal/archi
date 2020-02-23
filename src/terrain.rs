use image::GenericImageView;
use luminance::{
    context::GraphicsContext,
    linear::M44,
    pipeline::{BoundTexture, Pipeline, ShadingGate},
    pixel::{Floating, R32F},
    shader::program::{Program, Uniform},
    tess::Tess,
    texture::{Dim2, Flat, Texture},
};
use luminance_derive::UniformInterface;

#[derive(UniformInterface)]
pub struct TerrainShaderInterface {
    heightmap: Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    view_projection: Uniform<M44>,
}

type TerrainShader = Program<(), (), TerrainShaderInterface>;

pub struct Terrain {
    heightmap: Texture<Flat, Dim2, R32F>,
    shader: TerrainShader,
    tess: Tess,
}

impl Terrain {
    pub fn new(
        context: &mut impl GraphicsContext,
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
            let image =
                image.resize_exact(0x100, 0x100, image::FilterType::Triangle);
            let size = [image.width(), image.height()];
            let texture = Texture::new(context, size, 0, sampler).unwrap();

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
        &self,
        pipeline: &Pipeline,
        shader_gate: &mut ShadingGate<impl GraphicsContext>,
        view_projection: impl Into<M44>,
    ) {
        let Self {
            heightmap,
            shader,
            tess,
        } = self;
        shader_gate.shade(shader, |iface, mut render_gate| {
            iface.view_projection.update(view_projection.into());
            iface.heightmap.update(&pipeline.bind_texture(heightmap));
            render_gate.render(&Default::default(), |mut tess_gate| {
                tess_gate.render(tess);
            });
        })
    }
}
