extern crate sdl2;
extern crate gl;
extern crate num;
extern crate glm;
extern crate rand;
extern crate tobj;

// Shader sources
static VS_SRC: &'static str = include_str!("shaders/basicShader.vert");
static FS_SRC: &'static str = include_str!("shaders/basicShader.frag");

// Shader sources
static BUF_VIS_VS_SRC: &'static str = include_str!("shaders/bufVis.vert");
static BUF_VIS_FS_SRC: &'static str = include_str!("shaders/bufVis.frag");

static AMBIENT_VERT: &'static str = include_str!("shaders/ambient.light.vert");
static AMBIENT_FRAG: &'static str = include_str!("shaders/ambient.light.frag");

mod shader;
mod mesh;
mod camera;
mod model;
mod glerror;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_system = sdl_context.video().unwrap();
    sdl_context.mouse()
        .set_relative_mouse_mode(true);
    let window = video_system.window("archi", 800, 600)
        .fullscreen_desktop()
        .opengl()
        .build()
        .unwrap();
    let (width, height) = window.size();
    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_system.gl_get_proc_address(s) as * const _);

    let shader = {
        let sources = [
            (VS_SRC, gl::VERTEX_SHADER),
            (FS_SRC, gl::FRAGMENT_SHADER),
        ];
        let shader = shader::Shader::from_sources(&sources);
        shader
    };

    let ambient_shader = {
        let sources = [
            (AMBIENT_VERT, gl::VERTEX_SHADER),
            (AMBIENT_FRAG, gl::FRAGMENT_SHADER),
        ];
        let shader = shader::Shader::from_sources(&sources);
        shader
    };

    let ambient_mesh = {
        mesh::Mesh::ambient_light()
    };

    let ambient_color = glm::vec3(0.1, 0.1, 0.1);

    #[allow(unused_variables)]
    let buffer_vis_shader = {
        let sources = [
            (BUF_VIS_VS_SRC, gl::VERTEX_SHADER),
            (BUF_VIS_FS_SRC, gl::FRAGMENT_SHADER)
        ];
        let shader = shader::Shader::from_sources(&sources);
        shader
    };

    #[allow(unused_variables)]
    let (fbo, color_buffer, depth_buffer) = {
        use gl::types::*;
        use std::ptr;
        let mut fbo = 0;
        let mut color_buffer = 0;
        let mut depth_buffer = 0;
        unsafe {
            let width = width as i32;
            let height = height as i32;

            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            gl::GenTextures(1, &mut color_buffer);
            gl::BindTexture(gl::TEXTURE_2D, color_buffer);
            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           gl::RGB as GLint,
                           width,
                           height,
                           0,
                           gl::RGB,
                           gl::UNSIGNED_BYTE,
                           ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::NamedFramebufferTexture(fbo,
                                        gl::COLOR_ATTACHMENT0,
                                        color_buffer,
                                        0);

            gl::GenTextures(1, &mut depth_buffer);
            gl::BindTexture(gl::TEXTURE_2D, depth_buffer);
            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           gl::DEPTH_COMPONENT as i32,
                           width,
                           height,
                           0,
                           gl::DEPTH_COMPONENT,
                           gl::UNSIGNED_BYTE,
                           ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

            gl::NamedFramebufferTexture(fbo,
                                        gl::DEPTH_ATTACHMENT,
                                        depth_buffer,
                                        0);

            assert_no_gl_error!();
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("framebuffer not complete");
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        (fbo, color_buffer, depth_buffer)
    };


    let models = {
        use model::*;
        let meshes = model::from_obj("../../assets/models/spaceship/transport_shuttle.obj",
                                     0.1,
                                     true);
        meshes.into_iter()
            .map(|mesh| {
                let mesh = Box::new(mesh);
                let mesh = Box::leak(mesh);
                Model::new(mesh, num::one())
            })
            .collect::<Vec<Model>>()
    };

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
    }

    let mut camera = camera::Camera::persp(width as f32, height as f32, 0.1, 100.0);

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

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::Enable(gl::DEPTH_TEST);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        };
        shader.activate();
        for m in models.iter() {
            m.render(&camera);
        }

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Disable(gl::DEPTH_TEST);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::BindTexture(gl::TEXTURE_2D, color_buffer);
        }

        ambient_shader.activate();
        unsafe {
            gl::Uniform3fv(1, 1, &ambient_color[0]);
        }
        ambient_mesh.draw();
        }
        assert_no_gl_error!();

        window.gl_swap_window();
        previous_time = now;
    }
    println!("Quit");
}
