use crate::context::Context;
use luminance::{
    depth_test::DepthComparison,
    render_state::RenderState,
    shader::{Program, Uniform},
    shading_gate::ShadingGate,
    tess::{Mode, Tess, TessBuilder},
};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_gl::GL33;

#[derive(UniformInterface)]
pub struct SkyboxShaderInterface {
    view_projection: Uniform<[[f32; 4]; 4]>,
    exposure: Uniform<f32>,
}

type SkyboxShader = Program<GL33, (), (), SkyboxShaderInterface>;

pub struct Skybox {
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

        Ok(Self { shader, tess })
    }

    pub fn render(
        &mut self,
        shader_gate: &mut ShadingGate<Context>,
        view: glm::Mat4,
        projection: glm::Mat4,
        exposure: f32,
    ) {
        let Self { shader, tess } = self;

        let mut view = view;

        view[12] = 0.0;
        view[13] = 0.0;
        view[14] = 0.0;

        let view_projection = projection * view;

        shader_gate.shade(shader, |mut iface, uni, mut render_gate| {
            iface.set(&uni.view_projection, view_projection.into());
            iface.set(&uni.exposure, exposure);

            let state = RenderState::default()
                .set_depth_test(DepthComparison::LessOrEqual);
            render_gate.render(&state, |mut tess_gate| {
                tess_gate.render(&*tess);
            });
        })
    }
}
