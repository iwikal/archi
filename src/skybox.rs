use luminance::{
    context::GraphicsContext,
    linear::M44,
    pipeline::{BoundTexture, Pipeline, ShadingGate},
    pixel::{Floating, NormRGB8UI},
    shader::program::{Program, Uniform},
    tess::Tess,
    texture::{Cubemap, Flat, Texture},
};
use luminance_derive::{Semantics, Vertex, UniformInterface};
use std::path::Path;

#[derive(UniformInterface)]
pub struct SkyboxShaderInterface {
    cubemap: Uniform<&'static BoundTexture<'static, Flat, Cubemap, Floating>>,
    view_projection: Uniform<M44>,
}

type CubemapTexture = Texture<Flat, Cubemap, NormRGB8UI>;

type SkyboxShader = Program<(), (), SkyboxShaderInterface>;

pub struct Skybox {
    cubemap: CubemapTexture,
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

        let cubemap = {
            let size = 2048;
            use luminance::texture::{CubeFace, GenMipmaps};
            let texture = Texture::new(context, size, 0, &Default::default()).unwrap();

            let path = path.as_ref();
            for &(face, filename) in [
                (CubeFace::PositiveX, "positive_x.png"),
                (CubeFace::PositiveY, "positive_y.png"),
                (CubeFace::PositiveZ, "positive_z.png"),
                (CubeFace::NegativeX, "negative_x.png"),
                (CubeFace::NegativeY, "negative_y.png"),
                (CubeFace::NegativeZ, "negative_z.png"),
            ].iter() {
                let image = image::open(&path.join(filename)).unwrap_or_else(|e| {
                    panic!("could not open {:?}: {}", path, e);
                });

                match image {
                    image::ImageRgb8(..) => (),
                    _ => panic!("expected rgb8 ui format"),
                }

                texture.upload_part_raw(
                    GenMipmaps::Yes,
                    ([0, 0], face),
                    size,
                    &image.raw_pixels(),
                );
            }

            texture
        };

        let shader = crate::shader::from_strings(
            include_str!("../shaders/skybox.vert"),
            include_str!("../shaders/skybox.frag"),
        );
        Self {
            cubemap,
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
        let bound_cubemap = pipeline.bind_texture(&self.cubemap);
        shader_gate.shade(&self.shader, |render_gate, iface| {
            use luminance::{depth_test::DepthTest, render_state::RenderState};
            let state = RenderState::default()
                .set_depth_test(DepthTest::Off);
            render_gate.render(state, |tess_gate| {
                iface.view_projection.update(view_projection.into());
                iface.cubemap.update(&bound_cubemap);
                tess_gate.render(context, (&self.tess).into());
            });
        })
    }
}
