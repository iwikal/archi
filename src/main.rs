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
mod model;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context.video().unwrap();
    sdl_context.mouse()
        .set_relative_mouse_mode(true);
    let window = video_system.window("archi", 800, 600)
        .opengl()
        .build()
        .unwrap();
    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_system.gl_get_proc_address(s) as * const _);

    let m = {
        let shader = {
            let sources = [
                (VS_SRC, gl::VERTEX_SHADER),
                (FS_SRC, gl::FRAGMENT_SHADER)
            ];
            let shader = shader::Shader::from_sources(&sources);
            let shader = Box::new(shader);
            let shader = Box::leak(shader);
            shader
        };

        use mesh::*;
        use model::*;
        let mesh = Mesh::cube();
        let mesh = Box::new(mesh);
        let mesh = Box::leak(mesh);
        Model::new(mesh, shader)
    };

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    let mut camera = camera::Camera::new();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut should_quit = false;
    use std::time::{ Instant };
    let mut previous_time = Instant::now();
    while !should_quit {
        let now = Instant::now();
        let delta_t = now.duration_since(previous_time);
        let delta_seconds = delta_t.subsec_micros() as f32 / 1000_000.0;
        for e in event_pump.poll_iter() {
            use sdl2::event::Event;
            match e {
                Event::Quit {..} => { should_quit = true; },
                Event::KeyDown {scancode, ..} => {
                    if let Some(key) = scancode {
                        match key.name() {
                            "Escape" => { should_quit = true; },
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
        }

        camera.take_input(&event_pump, delta_seconds);

        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }

        {
            use model::Renderable;
            m.render(&camera);
        }

        window.gl_swap_window();
        previous_time = now;
    }
    println!("Quit");
}
