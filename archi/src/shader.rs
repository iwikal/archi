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
) -> Result<Stage, ()> {
    let body = context.shader_preprocessor.inner.expand(src.body).unwrap();

    Stage::new(context, ty, body).map_err(|err| {
        eprintln!(r#""{}": {}"#, src.name, err);
    })
}

type Stages = (Option<(Stage, Stage)>, Stage, Option<Stage>, Stage);

fn compile_stages(
    context: &mut Context,
    tess: Option<(&ShaderSource, &ShaderSource)>,
    vert: &ShaderSource,
    geom: Option<&ShaderSource>,
    frag: &ShaderSource,
) -> Result<Stages, ()> {
    let tess = tess
        .as_ref()
        .map(|(control_source, eval_source)| {
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
) -> Program<GL33, Sem, Out, Uni>
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
    )
    .expect("aborting due to previous errors");

    let tess_stage: Option<TessellationStages<Stage>> =
        tess_stage.as_ref().map(|(c, e)| tess_stages((c, e)));

    let geom_stage: Option<&Stage> = geom_stage.as_ref();

    let BuiltProgram { program, warnings } = context
        .new_shader_program()
        .from_stages(&vert_stage, tess_stage, geom_stage, &frag_stage)
        .unwrap_or_else(|error| {
            eprintln!("failed to build program with stages:");
            if let Some((control, eval)) = tess {
                eprintln!("  tessellation control:    {}", control.name);
                eprintln!("  tessellation evaluation: {}", eval.name);
            }
            eprintln!("  vertex stage:            {}", vert.name);
            if let Some(geom) = geom {
                eprintln!("  geometry stage:          {}", geom.name);
            }
            eprintln!("  fragment stage:          {}", frag.name);
            eprintln!("{}", error);
            panic!();
        });

    for warning in warnings {
        eprintln!("{}", warning);
    }

    program
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
