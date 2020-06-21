use anyhow::Context as _;
use crate::context::Context;
use luminance::context::GraphicsContext;
use luminance::shader::{BuiltProgram, Program};
use luminance::shader::{StageType, TessellationStages};
use luminance_gl::GL33;

type Stage = luminance::shader::Stage<GL33>;

pub struct Preprocessor {
    inner: glsl_include::Context<'static>,
}

impl Preprocessor {
    pub fn new() -> Self {
        let mut inner = glsl_include::Context::new();

        macro_rules! add_source {
            ($path:expr) => {
                let i = match $path.rfind("/") {
                    Some(i) => i + 1,
                    None => 0,
                };
                let name = &$path[i..];
                inner.include(name, include_str!($path));
            };
        }

        add_source!("./shaders/include/complex.glsl");
        add_source!("./shaders/include/tonemap.glsl");
        add_source!("./shaders/include/atmosphere.glsl");

        Self { inner }
    }
}

pub struct ShaderSource {
    pub name: &'static str,
    pub body: &'static str,
}

fn tess_stages<'a, S: ?Sized>(
    (control, evaluation): (&'a S, &'a S),
) -> TessellationStages<'a, S> {
    TessellationStages {
        control,
        evaluation,
    }
}

fn compile_stage(
    context: &mut Context,
    ty: StageType,
    src: &ShaderSource,
) -> anyhow::Result<Stage> {
    let body = context.shader_preprocessor.inner.expand(src.body)?;

    let stage = Stage::new(context, ty, body)
        .with_context(|| format!("failed to compile {} {}", ty, src.name))?;
    Ok(stage)
}

type Stages = (Option<(Stage, Stage)>, Stage, Option<Stage>, Stage);

fn compile_stages(
    context: &mut Context,
    tess: Option<(&ShaderSource, &ShaderSource)>,
    vert: &ShaderSource,
    geom: Option<&ShaderSource>,
    frag: &ShaderSource,
) -> anyhow::Result<Stages> {
    let tess = tess
        .as_ref()
        .map(|(control_source, eval_source)| -> anyhow::Result<_> {
            let tcs = compile_stage(
                context,
                StageType::TessellationControlShader,
                &control_source,
            );
            let tes = compile_stage(
                context,
                StageType::TessellationEvaluationShader,
                &eval_source,
            );
            Ok((tcs?, tes?))
        })
        .transpose();
    let vert = compile_stage(context, StageType::VertexShader, vert);
    let geom = geom
        .as_ref()
        .map(|src| compile_stage(context, StageType::GeometryShader, &src))
        .transpose();
    let frag = compile_stage(context, StageType::FragmentShader, frag);

    Ok((tess?, vert?, geom?, frag?))
}

pub fn from_sources<Sem, Out, Uni>(
    context: &mut Context,
    tess: Option<(ShaderSource, ShaderSource)>,
    vert: ShaderSource,
    geom: Option<ShaderSource>,
    frag: ShaderSource,
) -> anyhow::Result<Program<GL33, Sem, Out, Uni>>
where
    Sem: luminance::vertex::Semantics,
    Uni: luminance::shader::UniformInterface<GL33>,
{
    let (tess_stage, vert_stage, geom_stage, frag_stage) = compile_stages(
        context,
        tess.as_ref().map(|(c, s)| (c, s)),
        &vert,
        geom.as_ref(),
        &frag,
    )?;

    let tess_stage: Option<TessellationStages<Stage>> =
        tess_stage.as_ref().map(|(c, e)| tess_stages((c, e)));

    let geom_stage: Option<&Stage> = geom_stage.as_ref();

    let BuiltProgram { program, warnings } = context
        .new_shader_program()
        .from_stages(&vert_stage, tess_stage, geom_stage, &frag_stage)
        .with_context(|| {
            let mut buf = String::from("failed to link shader program:\n");
            if let Some((control, eval)) = tess {
                buf.push_str(&format!("  tessellation control:    {}", control.name));
                buf.push_str(&format!("  tessellation evaluation: {}", eval.name));
            }
            buf.push_str(&format!("  vertex stage:            {}", vert.name));
            if let Some(geom) = geom {
                buf.push_str(&format!("  geometry stage:          {}", geom.name));
            }
            buf.push_str(&format!("  fragment stage:          {}", frag.name));
            buf
        })?;

    for warning in warnings {
        eprintln!("{}", warning);
    }

    Ok(program)
}

#[macro_export]
macro_rules! shader_source {
    ($path:expr) => {
        crate::shader::ShaderSource {
            name: $path,
            body: include_str!($path),
        }
    };
}
