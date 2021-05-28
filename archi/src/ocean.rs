use crate::context::Context;
use luminance_derive::UniformInterface;
use luminance_front::{
    context::GraphicsContext,
    face_culling::{FaceCulling, FaceCullingMode},
    pipeline::{Pipeline, PipelineGate, TextureBinding},
    pixel::{Floating, RGB32F},
    render_state::RenderState,
    shader::{Program, Uniform},
    shading_gate::ShadingGate,
    tess::Tess,
    texture::{Dim2, Texture},
};

mod displacement;
mod gradient_jacobian;

use gradient_jacobian::DispGradJac;
use displacement::{
    Displacement, InitialDisplacement, InitialDisplacementTexture,
};

const QUAD_VS_SRC: crate::shader::ShaderSource =
    crate::shader_source!("./shaders/quad.vert");

const N: u32 = 0x200;

#[derive(UniformInterface)]
pub struct OceanShaderInterface {
    displacement_map: Uniform<TextureBinding<Dim2, Floating>>,
    #[uniform(unbound)]
    gradient_jacobian_map: Uniform<TextureBinding<Dim2, Floating>>,

    view_projection: Uniform<[[f32; 4]; 4]>,
    camera_offset: Uniform<[f32; 2]>,

    #[uniform(unbound)]
    sky_texture: Uniform<TextureBinding<Dim2, Floating>>,
    #[uniform(unbound)]
    camera_pos: Uniform<[f32; 3]>,
    #[uniform(unbound)]
    exposure: Uniform<f32>,
}

type OceanShader = Program<(), (), OceanShaderInterface>;

use crate::fft::{Fft, FftFramebuffer};
pub struct Ocean {
    pub init_disp_texture: InitialDisplacementTexture,
    pub displacement_freq: Displacement<N>,
    displacement_buffers: [FftFramebuffer; 3],
    pub fft: Fft,
    disp_grad_jac: DispGradJac<N>,
    shader: OceanShader,
    wireframe_shader: OceanShader,
    tess: Tess<(), u32>,
}

impl Ocean {
    pub fn new(context: &mut Context) -> anyhow::Result<Self> {
        let mut init_disp = InitialDisplacement::<N>::new(context)?;
        init_disp.render(&mut context.new_pipeline_gate())?;

        let displacement_freq = Displacement::<N>::new(context)?;
        let displacement_buffers = [
            Fft::framebuffer(context, N)?,
            Fft::framebuffer(context, N)?,
            Fft::framebuffer(context, N)?,
        ];
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

        let wireframe_shader = crate::shader::from_sources(
            context,
            Some((
                crate::shader_source!("./shaders/ocean.tesc"),
                crate::shader_source!("./shaders/ocean.tese"),
            )),
            crate::shader_source!("./shaders/ocean.vert"),
            Some(crate::shader_source!("./shaders/ocean.geom")),
            crate::shader_source!("./shaders/wireframe.frag"),
        )?;

        let tess = crate::grid::square_patch_grid(context, 0x100)?;

        let init_disp_texture = init_disp.into_texture();

        let disp_grad_jac = DispGradJac::new(context)?;

        Ok(Self {
            init_disp_texture,
            displacement_freq,
            displacement_buffers,
            fft,
            disp_grad_jac,
            shader,
            wireframe_shader,
            tess,
        })
    }

    pub fn simulate(
        &mut self,
        pipeline_gate: &mut PipelineGate,
        time: f32,
    ) -> anyhow::Result<OceanFrame> {
        let Self {
            init_disp_texture,
            displacement_freq,
            displacement_buffers,
            fft,
            disp_grad_jac,
            shader,
            wireframe_shader,
            tess,
        } = self;

        let (displacement_map, gradient_jacobian_map) = {
            let [hkt_x, hkt_y, hkt_z] = displacement_freq.render(
                pipeline_gate,
                time,
                init_disp_texture,
            )?;
            let [xmap, ymap, zmap] = displacement_buffers;
            let xmap = fft.render(pipeline_gate, hkt_x, xmap)?;
            let ymap = fft.render(pipeline_gate, hkt_y, ymap)?;
            let zmap = fft.render(pipeline_gate, hkt_z, zmap)?;
            disp_grad_jac.render(pipeline_gate, xmap, ymap, zmap)?
        };

        Ok(OceanFrame {
            shader,
            wireframe_shader,
            tess,
            displacement_map,
            gradient_jacobian_map,
        })
    }
}

pub struct OceanFrame<'a> {
    shader: &'a mut OceanShader,
    wireframe_shader: &'a mut OceanShader,
    tess: &'a mut Tess<(), u32>,
    pub displacement_map: &'a mut Texture<Dim2, RGB32F>,
    pub gradient_jacobian_map: &'a mut Texture<Dim2, RGB32F>,
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
        wireframe: bool,
    ) -> anyhow::Result<()> {
        let Self {
            shader,
            wireframe_shader,
            tess,
            displacement_map,
            gradient_jacobian_map,
        } = self;

        let displacement_map = pipeline.bind_texture(displacement_map)?;
        let gradient_jacobian_map = pipeline.bind_texture(gradient_jacobian_map)?;

        let shader = match wireframe {
            true => wireframe_shader,
            false => shader,
        };

        shader_gate.shade(shader, |mut iface, uni, mut render_gate| {
            iface.set(&uni.view_projection, view_projection.into());
            iface.set(&uni.displacement_map, displacement_map.binding());
            iface.set(&uni.gradient_jacobian_map, gradient_jacobian_map.binding());

            iface.set(&uni.camera_pos, camera_pos.into());
            if let Some(texture) = sky_texture {
                let texture = pipeline.bind_texture(texture)?;
                iface.set(&uni.sky_texture, texture.binding());
            }
            iface.set(&uni.exposure, exposure);

            render_gate.render(
                &RenderState::default().set_face_culling(None),
                |mut tess_gate| {
                    iface.set(&uni.camera_offset, [camera_pos.x, camera_pos.z]);
                    tess_gate.render(&**tess)
                },
            )
        })
    }
}
