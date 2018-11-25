extern crate sdl2;
extern crate gl;
extern crate num;
extern crate glm;

// Shader sources
static VS_SRC: &'static str = include_str!("shaders/basicShader.vert");
static FS_SRC: &'static str = include_str!("shaders/basicShader.frag");

mod shader;
mod mesh;
mod camera;

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
        let program = shader::Shader::from_sources(&[
           (VS_SRC, gl::VERTEX_SHADER),
           (FS_SRC, gl::FRAGMENT_SHADER)
        ]);

        use mesh::Vertex;
        use glm::vec3;
        let vertices = [
            Vertex { position: vec3( 0.0,  1.0, 0.0) },
            Vertex { position: vec3( 0.5, -0.5, 0.0) },
            Vertex { position: vec3(-0.5, -0.5, 0.0) }
        ];
        let indices = [0, 1, 2];
        mesh::new(&vertices, &indices, program)
    };

    let mut camera = camera::Camera::new();

    let pv_location = m.program.get_location("projection_view");

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut should_quit = false;
    while !should_quit {
        for e in event_pump.poll_iter() {
            use sdl2::event::Event;
            match e {
                Event::Quit {..} => break,
                Event::KeyDown {scancode, ..} => {
                    match scancode {
                        Some(key) => {
                            let name = key.name();
                            if name == "Escape" { should_quit = true; }
                        },
                        _ => ()
                    }
                },
                _ => {}
            }
        }

        camera.take_input(&event_pump);

        let view = camera.view();
        let projection = camera.projection();
        let projection_view = projection * view;

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UniformMatrix4fv(pv_location,
                                 1,
                                 gl::FALSE,
                                 &(projection_view[0][0]));
        }

        {
            use mesh::Renderable;
            m.render();
        }

        window.gl_swap_window();
    }
    println!("Quit");
}
