use luminance::{
    context::GraphicsContext,
    linear::M44,
    pipeline::{BoundTexture, Pipeline, ShadingGate},
    pixel::{Floating, RGB32F},
    shader::program::{Program, Uniform},
    tess::Tess,
    texture::{Dim2, Flat, Texture},
};
use luminance_derive::{Semantics, Vertex, UniformInterface};
use std::path::Path;

#[derive(UniformInterface)]
pub struct SkyboxShaderInterface {
    box_face: Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    view_projection: Uniform<M44>,
}

type CubefaceTexture = Texture<Flat, Dim2, RGB32F>;

type SkyboxShader = Program<(), (), SkyboxShaderInterface>;

pub struct Skybox {
    negative_x: CubefaceTexture,
    negative_y: CubefaceTexture,
    negative_z: CubefaceTexture,
    positive_x: CubefaceTexture,
    positive_y: CubefaceTexture,
    positive_z: CubefaceTexture,
    tess: Tess,
    shader: SkyboxShader,
}

#[derive(Debug, Clone, Copy, Semantics)]
pub enum Semantics {
    #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "uv", repr = "[f32; 2]", wrapper = "VertexUv")]
    Uv,
}

#[derive(Vertex)]
#[vertex(sem = "Semantics")]
#[allow(unused)]
struct CubeVertex {
    position: VertexPosition,
    uv: VertexUv,
}

impl Skybox {
    pub fn new(
        context: &mut impl GraphicsContext,
        path: impl AsRef<Path>,
    ) -> Self {
        let tess = {
            use luminance::tess::{TessBuilder, Mode};
            let (vertices, indices) = {
                let n_vertices = 24;
                let n_indices = 36;
                let mut vertices = Vec::with_capacity(n_vertices);
                let mut indices = Vec::with_capacity(n_indices);
                for dimension in 0..3_u8 {
                    for z in 0..2_u8 {
                        let n = vertices.len() as u32;
                        for x in 0..2_u8 {
                            for y in 0..2_u8 {
                                let mut position = [
                                    f32::from(x) * 2.0 - 1.0,
                                    f32::from(y) * 2.0 - 1.0,
                                    f32::from(z) * 2.0 - 1.0,
                                ];
                                position.rotate_left(usize::from(dimension));
                                if z != 0 {
                                    position.reverse();
                                }
                                vertices.push(CubeVertex {
                                    position: VertexPosition::new(position),
                                    uv: VertexUv::new([f32::from(x), f32::from(y)]),
                                });
                            }
                        }
                        indices.push(n);
                        indices.push(1 + n);
                        indices.push(2 + n);
                        indices.push(1 + n);
                        indices.push(2 + n);
                        indices.push(3 + n);
                    }
                }
                // assert_eq!(vertices.len(), n_vertices);
                // assert_eq!(indices.len(), n_indices);
                (vertices, indices)
            };

            TessBuilder::new(context)
                .set_mode(Mode::Triangle)
                .add_vertices(&vertices)
                .set_indices(&indices)
                .build()
                .unwrap()
        };

        let mut load_texture = |name| {
            use luminance::texture::{GenMipmaps, MagFilter, MinFilter, Sampler};
            use image::GenericImageView;

            let mut sampler = Sampler::default();
            sampler.mag_filter = MagFilter::Nearest;
            sampler.min_filter = MinFilter::Nearest;
            let path = path.as_ref().join(name);
            let image = image::open(&path).unwrap_or_else(|e| {
                panic!("could not open {:?}: {}", path, e);
            });
            let size = [image.width(), image.height()];
            let texture = Texture::new(context, size, 0, &sampler).unwrap();

            let pixels: Vec<_> = image
                .pixels()
                .map(|(_x, _y, p)| {
                    use image::Pixel;
                    let image::Rgb([r, g, b]) = p.to_rgb();
                    (
                        f32::from(r) / 255.0,
                        f32::from(g) / 255.0,
                        f32::from(b) / 255.0,
                    )
                })
                .collect();

            texture.upload(GenMipmaps::Yes, &pixels);
            texture
        };

        let shader = crate::shader::from_strings(
            include_str!("../shaders/skybox.vert"),
            include_str!("../shaders/skybox.frag"),
        );
        Self {
            negative_x: load_texture("negative_x.png"),
            negative_y: load_texture("negative_y.png"),
            negative_z: load_texture("negative_z.png"),
            positive_x: load_texture("positive_x.png"),
            positive_y: load_texture("positive_y.png"),
            positive_z: load_texture("positive_z.png"),
            shader,
            tess,
        }
    }

    pub fn render(
        &self,
        context: &mut impl GraphicsContext,
        pipeline: &Pipeline,
        shader_gate: &ShadingGate,
        camera: &crate::camera::Camera,
    ) {
        let view_projection = camera.projection() * camera.orientation();
        let textures = [
            pipeline.bind_texture(&self.negative_x),
            pipeline.bind_texture(&self.negative_y),
            pipeline.bind_texture(&self.negative_z),
            pipeline.bind_texture(&self.positive_x),
            pipeline.bind_texture(&self.positive_y),
            pipeline.bind_texture(&self.positive_z),
        ];
        shader_gate.shade(&self.shader, |render_gate, iface| {
            use luminance::{depth_test::DepthTest, render_state::RenderState};
            let state = RenderState::default()
                .set_depth_test(DepthTest::Off);
            render_gate.render(state, |tess_gate| {
                iface.view_projection.update(view_projection.into());
                iface.box_face.update(&textures[0]);
                tess_gate.render(context, (&self.tess).into());
            });
        })
    }
}
