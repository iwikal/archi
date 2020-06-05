use crate::context::Context;
use luminance::{
    pipeline::{Pipeline, TextureBinding},
    pixel::{Floating, RGB32F},
    shader::{Program, Uniform},
    shading_gate::ShadingGate,
    tess::{Mode, Tess, TessBuilder},
    texture::{Dim2, Texture},
};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_gl::GL33;

#[derive(UniformInterface)]
pub struct SkyboxShaderInterface {
    hdri: Uniform<TextureBinding<Dim2, Floating>>,
    view_projection: Uniform<[[f32; 4]; 4]>,
    exposure: Uniform<f32>,
}

type MapTexture = Texture<GL33, Dim2, RGB32F>;

type SkyboxShader = Program<GL33, (), (), SkyboxShaderInterface>;

pub struct Skybox {
    hdri: MapTexture,
    tess: Tess<GL33, CubeVertex, u32>,
    shader: SkyboxShader,
}

#[derive(Debug, Clone, Copy, Semantics)]
pub enum Semantics {
    #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "uv", repr = "[f32; 2]", wrapper = "VertexUv")]
    Uv,
}

#[derive(Clone, Copy, Vertex)]
#[vertex(sem = "Semantics")]
#[allow(unused)]
struct CubeVertex {
    position: VertexPosition,
    uv: VertexUv,
}

#[derive(Default)]
pub struct ImageData {
    pub width: usize,
    pub height: usize,
    pub data: Vec<(f32, f32, f32)>,
}

impl Skybox {
    pub fn load_image() -> Result<ImageData, String> {
        let bytes: &[u8] = include_bytes!("../assets/colorful_studio_8k.hdr");

        let image = hdrldr::load(bytes).map_err(|e| {
            let err_str = match e {
                hdrldr::LoadError::Io(e) => format!("{}", e),
                hdrldr::LoadError::FileFormat => String::from("invalid file"),
                hdrldr::LoadError::Rle => {
                    String::from("invalid run-length encoding")
                }
            };
            format!("could not load skybox: {}", err_str)
        })?;

        let hdrldr::Image {
            data,
            width,
            height,
        } = image;

        let data: Vec<_> = data
            .into_iter()
            .map(|hdrldr::RGB { r, g, b }| (r, g, b))
            .collect();

        Ok(ImageData {
            width,
            height,
            data,
        })
    }

    pub fn new(context: &mut Context, image: ImageData) -> Self {
        let tess = {
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
                                    uv: VertexUv::new([
                                        f32::from(x),
                                        f32::from(y),
                                    ]),
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
                .set_vertices(vertices)
                .set_indices(indices)
                .build()
                .unwrap()
        };

        let shader = crate::shader::from_strings(
            context,
            None,
            include_str!("./shaders/skybox.vert"),
            include_str!("./shaders/skybox.frag"),
        );

        let hdri = {
            let mut texture = Texture::new(
                context,
                [image.width as u32, image.height as u32],
                0,
                Default::default(),
            )
            .map_err(|e| e.to_string())
            .unwrap();

            texture
                .upload(luminance::texture::GenMipmaps::Yes, &image.data)
                .map_err(|e| e.to_string())
                .unwrap();

            texture
        };

        Self { hdri, shader, tess }
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline<GL33>,
        shader_gate: &mut ShadingGate<Context>,
        view: glm::Mat4,
        projection: glm::Mat4,
        exposure: f32,
    ) {
        let Self { shader, hdri, tess } = self;

        let mut view = view;

        view[12] = 0.0;
        view[13] = 0.0;
        view[14] = 0.0;

        let view_projection = projection * view;

        let bound_hdri = pipeline.bind_texture(hdri).unwrap();
        shader_gate.shade(shader, |mut iface, uni, mut render_gate| {
            use luminance::{
                depth_test::DepthComparison, render_state::RenderState,
            };
            iface.set(&uni.view_projection, view_projection.into());
            iface.set(&uni.hdri, bound_hdri.binding());
            iface.set(&uni.exposure, exposure);

            let state = RenderState::default()
                .set_depth_test(DepthComparison::LessOrEqual);
            render_gate.render(&state, |mut tess_gate| {
                tess_gate.render(&*tess);
            });
        })
    }

    pub fn texture(&mut self) -> &mut MapTexture {
        &mut self.hdri
    }
}
