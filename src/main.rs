extern crate gl;
extern crate glm;
extern crate gltf;
extern crate lodepng;
extern crate num;
extern crate rand;
extern crate rgb;
extern crate sdl2;
extern crate specs;
extern crate specs_hierarchy;
extern crate tobj;

mod camera;
mod glerror;
mod mesh;
mod model;
mod renderer;
mod shader;
mod world;

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let video_system = sdl_context.video().unwrap();
    sdl_context.mouse().set_relative_mouse_mode(true);
    let gl_attr = video_system.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 3);
    gl_attr.set_framebuffer_srgb_compatible(true);
    gl_attr.set_context_flags().debug().set();

    dbg!(video_system.current_video_driver());

    let window = video_system
        .window("archi", 800, 600)
        .fullscreen_desktop()
        .opengl()
        .build()
        .unwrap();
    let (width, height) = window.size();
    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_system.gl_get_proc_address(s) as *const _);

    {
        use glerror::*;
        debug_messages(GlDebugSeverity::Low);
    }

    unsafe {
        use std::ffi::*;
        eprintln!(
            "GL_VENDOR:	{:?}",
            CStr::from_ptr(gl::GetString(gl::VENDOR) as *const i8)
        );
        eprintln!(
            "GL_RENDERER:	{:?}",
            CStr::from_ptr(gl::GetString(gl::RENDERER) as *const i8)
        );
        eprintln!(
            "GL_VERSION:	{:?}",
            CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8)
        );
    }

    let mut renderer = renderer::Renderer::new(width as i32, height as i32);

    let brightness = 1.0 / 2048.0;
    let ambient_color = glm::vec3(brightness, brightness, brightness);

    let point_lights = {
        use glm::vec3;
        use renderer::PointLight as Light;
        [
            Light {
                radius: 4.0,
                position: vec3(-1.3, 2.5, 0.),
                color: vec3(0.5, 0.5, 0.4) / 16.0,
            },
            Light {
                radius: 4.0,
                position: vec3(0., 2.5, 0.),
                color: vec3(0.5, 0.5, 0.4) / 16.0,
            },
            Light {
                radius: 4.0,
                position: vec3(1.3, 2.5, 0.),
                color: vec3(0.5, 0.5, 0.4) / 16.0,
            },
            Light {
                radius: 4.0,
                position: vec3(0., -0.5, 2.),
                color: vec3(0.5, 0.5, 1.) / 16.0,
            },
            Light {
                radius: 4.0,
                position: vec3(0., -0.5, -2.),
                color: vec3(0.5, 0.5, 1.) / 16.0,
            },
            Light {
                radius: 4.0,
                position: vec3(3.5, 4.3, 1.),
                color: vec3(1., 0., 0.) / 16.0,
            },
        ]
    };

    let dir_lights = {
        use glm::vec3;
        use renderer::DirectionalLight as Light;
        [Light {
            direction: vec3(-1., -1., -1.),
            color: vec3(0.5, 0.5, 0.75) / 512.0,
        }]
    };

    let gltf = gltf::Gltf::open(
        "../../assets/models/glTF-Sample-Models/2.0/TriangleWithoutIndices/glTF/TriangleWithoutIndices.gltf"
    ).unwrap();

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            println!("{:#?}", node);
        }
    }

    let models = {
        use model::*;
        let meshes = model::from_obj("assets/models/spaceship/transport_shuttle.obj", 1.0, true);
        meshes
            .into_iter()
            .map(|mesh| {
                let mesh = Box::new(mesh);
                let mesh = Box::leak(mesh);
                Model::new(mesh, num::one())
            })
            .collect::<Vec<Model>>()
    };

    world::load_world();

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::BlendFunc(gl::ONE, gl::ONE);
    }

    let mut camera = camera::Camera::persp(width as f32, height as f32, 0.1, 100.0);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut should_quit = false;
    use std::time::Instant;
    let mut previous_time = Instant::now();

    use std::num::Wrapping;
    let mut frame_count = Wrapping(0);
    let mut temporal_dither = true;
    while !should_quit {
        let now = Instant::now();
        let delta_t = now.duration_since(previous_time);
        let delta_seconds = delta_t.subsec_micros() as f32 / 1000_000.0;
        for e in event_pump.poll_iter() {
            use sdl2::event::Event;
            match e {
                Event::Quit { .. } => {
                    should_quit = true;
                }
                Event::KeyDown { scancode, .. } => {
                    if let Some(key) = scancode {
                        match key.name() {
                            "Escape" => {
                                should_quit = true;
                            }
                            "F" => {
                                temporal_dither = !temporal_dither;
                            }
                            "Q" => {
                                renderer.color_depth = 256;
                            }
                            "E" => {
                                renderer.color_depth = 2;
                            }
                            _ => {}
                        }
                    }
                }
                Event::MouseWheel { y, .. } => {
                    renderer.res_factor = if y > 0 {
                        let f = renderer.res_factor * 2;
                        if f > 32 {
                            32
                        } else {
                            f
                        }
                    } else {
                        let f = renderer.res_factor / 2;
                        if f < 1 {
                            1
                        } else {
                            f
                        }
                    };
                }
                _ => {}
            }
        }

        camera.take_input(&event_pump, delta_seconds);

        renderer.render(
            frame_count.0,
            &camera,
            &models,
            ambient_color,
            &dir_lights,
            &point_lights,
        );
        window.gl_swap_window();
        previous_time = now;
        if temporal_dither {
            frame_count += Wrapping(1);
        }
    }
    println!("Quit");
}
