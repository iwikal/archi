use luminance::{
    context::GraphicsContext,
    linear::M44,
    pipeline::{BoundTexture, Pipeline, ShadingGate},
    pixel::{Floating, R32F},
    render_state::RenderState,
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

impl TerrainShaderInterface {
    pub fn set_view_projection(&self, value: M44) {
        self.view_projection.update(value);
    }

    pub fn set_heightmap(&self, value: &BoundTexture<Flat, Dim2, Floating>) {
        self.heightmap.update(value);
    }
}

type TerrainShader = Program<(), (), TerrainShaderInterface>;

pub struct Terrain {
    heightmap: Texture<Flat, Dim2, R32F>,
    shader: TerrainShader,
    tess: Tess,
}

impl Terrain {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let heightmap = {
            use luminance::texture::{MagFilter, MinFilter, Sampler};
            let mut sampler = Sampler::default();
            sampler.mag_filter = MagFilter::Nearest;
            sampler.min_filter = MinFilter::Nearest;
            let texture = Texture::new(context, [0, 0], 0, &sampler).unwrap();

            texture
        };
        let tess = crate::attributeless_grid(context, 0x100);
        let shader = crate::shader::from_strings(
            include_str!("../shaders/terrain.vert"),
            include_str!("../shaders/terrain.frag"),
        );
        Self {
            heightmap,
            shader,
            tess,
        }
    }

    pub fn render(
        &self,
        context: &mut impl GraphicsContext,
        pipeline: &Pipeline,
        shader_gate: &ShadingGate,
        view_projection: impl Into<M44>,
    ) {
        let Self {
            heightmap,
            shader,
            tess,
        } = self;
        shader_gate.shade(shader, |render_gate, iface| {
            iface.set_view_projection(view_projection.into());
            iface.set_heightmap(&pipeline.bind_texture(heightmap));
            render_gate.render(RenderState::default(), |tess_gate| {
                tess_gate.render(context, tess.into());
            });
        })
    }
}
