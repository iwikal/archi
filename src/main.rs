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
mod material;
mod mesh;
mod renderer;
mod shader;
mod world;

use camera::Camera;
use renderer::Renderer;
use specs::prelude::*;

fn main() {
    let mut args = std::env::args();
    let model_path = {
        let _exe_path = args.next().expect("Must have argv[0]");
        args.next().unwrap_or(
            "assets/models/2.0/FlightHelmet/glTF/FlightHelmet.gltf".to_owned(),
        )
    };
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

    let renderer = renderer::Renderer::new(width as i32, height as i32);

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::BlendFunc(gl::ONE, gl::ONE);
    }

    let camera = Camera::persp(width as f32, height as f32, 0.1, 100.0);

    let (world, mut dispatcher) =
        world::load_world(model_path, renderer, camera);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut should_quit = false;
    use std::time::Instant;
    let mut previous_time = Instant::now();

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
                                let mut renderer =
                                    world.write_resource::<Renderer>();
                                renderer.temporal_dither =
                                    !renderer.temporal_dither;
                            }
                            "Q" => {
                                world
                                    .write_resource::<Renderer>()
                                    .color_depth = 256;
                            }
                            "E" => {
                                world
                                    .write_resource::<Renderer>()
                                    .color_depth = 2;
                            }
                            _ => {}
                        }
                    }
                }
                Event::MouseWheel { y, .. } => {
                    let mut renderer = world.write_resource::<Renderer>();
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
        world
            .write_resource::<Camera>()
            .take_input(&event_pump, delta_seconds);

        dispatcher.dispatch_seq(&world);

        window.gl_swap_window();
        previous_time = now;
    }
    eprintln!("Quit");
}
