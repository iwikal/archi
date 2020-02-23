use luminance::shader::program::{BuiltProgram, Program};
use luminance::shader::stage::{Stage, Type};

pub struct ShaderSource {
    pub name: &'static str,
    pub body: &'static str,
}

pub fn from_strings<S, Out, Uni>(
    tess: Option<(&str, &str)>,
    vert: &str,
    frag: &str,
) -> Program<S, Out, Uni>
where
    S: luminance::vertex::Semantics,
    Uni: luminance::shader::program::UniformInterface,
{
    let BuiltProgram { program, warnings } =
        Program::from_strings(tess, vert, None, frag).unwrap_or_else(|error| {
            eprintln!("{}", error);
            panic!("{:?}", (tess, vert, frag));
        });

    for warning in warnings {
        eprintln!("{}", warning);
    }

    program
}

fn compile_stage(ty: Type, src: &ShaderSource) -> Result<Stage, ()> {
    Stage::new(ty, src.body).map_err(|err| {
        eprintln!(r#""{}": {}"#, src.name, err);
    })
}

type Stages = (Option<(Stage, Stage)>, Stage, Option<Stage>, Stage);

fn compile_stages(
    tess: &Option<(ShaderSource, ShaderSource)>,
    vert: &ShaderSource,
    geom: &Option<ShaderSource>,
    frag: &ShaderSource,
) -> Result<Stages, ()> {
    let tess = tess
        .as_ref()
        .map(|(control_source, eval_source)| {
            let tcs =
                compile_stage(Type::TessellationControlShader, &control_source);
            let tes =
                compile_stage(Type::TessellationEvaluationShader, &eval_source);
            Ok((tcs?, tes?))
        })
        .transpose();
    let vert = compile_stage(Type::VertexShader, vert);
    let geom = geom
        .as_ref()
        .map(|src| compile_stage(Type::GeometryShader, &src))
        .transpose();
    let frag = compile_stage(Type::FragmentShader, frag);

    Ok((tess?, vert?, geom?, frag?))
}

pub fn from_sources<S, Out, Uni>(
    tess: Option<(ShaderSource, ShaderSource)>,
    vert: ShaderSource,
    geom: Option<ShaderSource>,
    frag: ShaderSource,
) -> Program<S, Out, Uni>
where
    S: luminance::vertex::Semantics,
    Uni: luminance::shader::program::UniformInterface,
{
    let (tess_stage, vert_stage, geom_stage, frag_stage) =
        compile_stages(&tess, &vert, &geom, &frag)
            .expect("aborting due to previous errors");

    let BuiltProgram { program, warnings } = Program::from_stages(
        tess_stage.as_ref().map(|(c, e)| (c, e)),
        &vert_stage,
        geom_stage.as_ref(),
        &frag_stage,
    )
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
