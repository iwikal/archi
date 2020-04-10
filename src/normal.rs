use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{BoundTexture, Builder},
    pixel::{Pixel, RGB32F},
    shader::program::{Program, Uniform, UniformInterface},
    tess::Tess,
    tess::{Mode, TessBuilder},
    texture::{Dim2, Flat, Texture},
};

const QUAD_VS_SRC: crate::shader::ShaderSource =
    crate::shader_source!("./shaders/quad.vert");

pub struct NormalShaderInterface<P>
where
    P: Pixel,
    P::SamplerType: 'static,
{
    heightmap:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, P::SamplerType>>,
}

impl<P: Pixel> UniformInterface for NormalShaderInterface<P> {
    fn uniform_interface(
        builder: &mut luminance::shader::program::UniformBuilder,
        _: (),
    ) -> Result<Self, luminance::shader::program::ProgramError> {
        let heightmap = builder.ask_unbound("heightmap");
        let iface = NormalShaderInterface { heightmap };
        Ok(iface)
    }
}

type NormalShader<P> = Program<(), (), NormalShaderInterface<P>>;
type NormalFramebuffer = Framebuffer<Flat, Dim2, RGB32F, ()>;
pub type NormalTexture = Texture<Flat, Dim2, RGB32F>;

pub struct NormalGenerator<P>
where
    P: Pixel,
    P::SamplerType: 'static,
{
    tess: Tess,
    shader: NormalShader<P>,
    framebuffer: NormalFramebuffer,
}

impl<P: Pixel> NormalGenerator<P> {
    pub fn new(context: &mut impl GraphicsContext, size: [u32; 2]) -> Self {
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
        let framebuffer = Framebuffer::new(context, size, 0, sampler)
            .expect("framebuffer creation");

        Self {
            shader,
            tess,
            framebuffer,
        }
    }

    pub fn render(
        &self,
        builder: &mut Builder<impl GraphicsContext>,
        heightmap: &Texture<Flat, Dim2, P>,
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
