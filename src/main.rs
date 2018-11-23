extern crate sdl2;
extern crate gl;

// Shader sources
static VS_SRC: &'static str = include_str!("shaders/basicShader.vert");
static FS_SRC: &'static str = include_str!("shaders/basicShader.frag");

mod shader;
mod mesh;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context.video().unwrap();

    let window = video_system.window("archi", 800, 600)
        .opengl()
        .build()
        .unwrap();
    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_system.gl_get_proc_address(s) as * const _);

    let m = {
        use mesh::{ Vertex, new };
        let program = shader::link(
            &[
            shader::compile(VS_SRC, gl::VERTEX_SHADER),
            shader::compile(FS_SRC, gl::FRAGMENT_SHADER)
            ]);
        let vertices = [
            Vertex { position: glm::vec3( 0.0,  1.0, 0.0) },
            Vertex { position: glm::vec3( 0.5, -0.5, 0.0) },
            Vertex { position: glm::vec3(-0.5, -0.5, 0.0) }
        ];
        let indices = [0, 1, 2];
        new(&vertices, &indices, program)
    };

    let mut event_pump = sdl_context.event_pump().unwrap();
    for e in event_pump.wait_iter() {
        use sdl2::event::Event;
        match e {
            Event::Quit {..} => break,
            Event::KeyDown {keycode, ..} => {
                match keycode {
                    Some(k) => if k.name() == "Escape" { break; },
                    _ => ()
                }
            },
            _ => {}
        }
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }

        {
            use mesh::Renderable;
            m.render();
        }

        window.gl_swap_window();
    }
    println!("Quit");
}
