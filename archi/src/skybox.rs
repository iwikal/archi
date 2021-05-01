use crate::context::Context;
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::{
    depth_test::DepthComparison,
    pipeline::{Pipeline, TextureBinding},
    pixel::{Floating, RGB32F},
    render_state::RenderState,
    shader::{Program, Uniform},
    shading_gate::ShadingGate,
    tess::{Mode, Tess, TessBuilder},
    texture::{Dim2, Sampler, Texture, Wrap},
};

#[derive(UniformInterface)]
pub struct SkyboxShaderInterface {
    sky_texture: Uniform<TextureBinding<Dim2, Floating>>,
    view_projection: Uniform<[[f32; 4]; 4]>,
    exposure: Uniform<f32>,
}

type SkyboxShader = Program<(), (), SkyboxShaderInterface>;

pub struct Skybox {
    pub sky_texture: Texture<Dim2, RGB32F>,
    tess: Tess<CubeVertex, u32>,
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
struct CubeVertex {
    #[allow(unused)]
    position: VertexPosition,
    #[allow(unused)]
    uv: VertexUv,
}

#[derive(Default)]
pub struct ImageData {
    pub width: usize,
    pub height: usize,
    pub data: Vec<(f32, f32, f32)>,
}

fn load_sky(context: &mut Context) -> anyhow::Result<Texture<Dim2, RGB32F>> {
    let file: &[u8] = include_bytes!("../assets/colorful_studio_8k.hdr");
    let mut loader = radiant::Loader::new(file)?.scanlines();

    let width = loader.width as u32;
    let height = loader.height as u32;

    let mut buf = vec![radiant::Rgb::zero(); loader.width];

    let mut texture = Texture::new(
        context,
        [width, height],
        0,
        Sampler {
            wrap_r: Wrap::Repeat,
            wrap_s: Wrap::ClampToEdge,
            ..Default::default()
        },
    )?;

    for y in 0..height {
        loader.read_scanline(&mut buf)?;
        texture.upload_part_raw(
            luminance::texture::GenMipmaps::No,
            [0, y],
            [width, 1],
            bytemuck::cast_slice(&buf),
        )?;
    }

    Ok(texture)
}

impl Skybox {
    pub fn new(context: &mut Context) -> anyhow::Result<Self> {
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
                assert_eq!(vertices.len(), n_vertices);
                assert_eq!(indices.len(), n_indices);
                (vertices, indices)
            };

            TessBuilder::new(context)
                .set_mode(Mode::Triangle)
                .set_vertices(vertices)
                .set_indices(indices)
                .build()
                .unwrap()
        };

        let shader = crate::shader::from_sources(
            context,
            None,
            crate::shader_source!("./shaders/skybox.vert"),
            None,
            crate::shader_source!("./shaders/skybox.frag"),
        )?;

        let sky_texture = load_sky(context)?;

        Ok(Self {
            sky_texture,
            tess,
            shader,
        })
    }

    pub fn render(
        &mut self,
        pipeline: &mut Pipeline,
        shader_gate: &mut ShadingGate,
        view: glm::Mat4,
        projection: glm::Mat4,
        exposure: f32,
    ) -> anyhow::Result<()> {
        let Self {
            shader,
            tess,
            sky_texture,
        } = self;

        let mut view = view;

        view[12] = 0.0;
        view[13] = 0.0;
        view[14] = 0.0;

        let view_projection = projection * view;
        let sky_texture = pipeline.bind_texture(sky_texture)?;

        shader_gate.shade(shader, |mut iface, uni, mut render_gate| {
            iface.set(&uni.sky_texture, sky_texture.binding());
            iface.set(&uni.view_projection, view_projection.into());
            iface.set(&uni.exposure, exposure);

            let state = RenderState::default()
                .set_depth_test(DepthComparison::LessOrEqual);
            render_gate.render(&state, |mut tess_gate| tess_gate.render(&*tess))
        })
    }
}
