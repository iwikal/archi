use luminance::context::GraphicsContext;
extern crate nalgebra_glm as glm;

mod shader;

mod camera;
mod context;
// mod debug;
mod fft;
mod glerror;
mod grid;
mod ocean;
mod skybox;
mod terrain;

pub use context::Context;

fn main() {
    eprintln!("running!");

    let context = &mut context::Context::new();

    let (width, height) = context.surface.window().size();

    let mut event_pump = context.surface.sdl().event_pump().unwrap();
    let mut back_buffer = context.surface.back_buffer().unwrap();

    let mut camera = {
        let aspect = width as f32 / height as f32;
        let fov = 1.1;
        let near = 0.1;
        camera::Camera::persp(aspect, fov, near)
    };

    let mut skybox_index = 0;

    let mut skyboxes: Vec<_> = {
        let paths = ["assets/colorful_studio_8k.hdr"];

        paths
            .iter()
            .map(|path| skybox::Skybox::new(context, path))
            .collect()
    };

    let mut ocean = ocean::Ocean::new(context);
    let mut terrain = terrain::Terrain::new(context, "assets/heightmap.png");

    let mut exposure = 0.2;

    let mut render_stuff = false;

    use std::time::Instant;
    let start = Instant::now();
    let mut previous_frame_start = start;
    'game_loop: loop {
        let current_frame_start = Instant::now();
        let delta_t = current_frame_start - previous_frame_start;

        let mut resize = None;
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => {
                    break 'game_loop;
                }
                Event::KeyDown { scancode, .. } => {
                    use sdl2::keyboard::Scancode::*;

                    match scancode {
                        Some(Escape) => break 'game_loop,
                        Some(E) => render_stuff = !render_stuff,
                        Some(Num1) => skybox_index = 0,
                        Some(Num2) => skybox_index = 1,
                        _ => {}
                    }
                }
                Event::MouseMotion { xrel, yrel, .. } => {
                    camera.mouse_moved(xrel as f32, yrel as f32);
                }
                Event::MouseWheel { y, .. } => {
                    exposure *= 2.0_f32.powi(y);
                }
                Event::Window { win_event, .. } => {
                    use sdl2::event::WindowEvent;
                    if let WindowEvent::SizeChanged(width, height) = win_event {
                        resize = Some([width, height]);
                    }
                }
                _ => {}
            }
        }

        if let Some(_) = resize {
            back_buffer = context.surface.back_buffer().unwrap();
        }

        camera.take_input(&event_pump);
        let delta_f = delta_t.as_micros() as f32 / 1_000_000.0;
        camera.physics_tick(delta_f);

        let skybox = &mut skyboxes[skybox_index];

        let mut pipeline_gate = context.pipeline_gate();

        let duration = current_frame_start - start;
        let f_time = duration.as_secs_f32();

        let mut ocean_frame = None;
        if render_stuff {
            ocean_frame = Some(ocean.simulate(&mut pipeline_gate, f_time));
        }

        use luminance::pipeline::PipelineState;

        pipeline_gate
            .pipeline(
                &back_buffer,
                &PipelineState::new().set_clear_color([0.1, 0.2, 0.3, 1.0]),
                |pipeline, mut shader_gate| {
                    let view = camera.view();
                    let projection = camera.projection();

                    let view_projection = projection * view;

                    if let Some(mut ocean_frame) = ocean_frame {
                        ocean_frame.render(
                            &pipeline,
                            &mut shader_gate,
                            view_projection,
                            camera.position(),
                            skybox.texture(),
                            exposure,
                        );
                    }

                    if render_stuff {
                        terrain.render(
                            &pipeline,
                            &mut shader_gate,
                            view_projection,
                        );
                    }

                    skybox.render(
                        &pipeline,
                        &mut shader_gate,
                        view,
                        projection,
                        exposure,
                    );
                },
            )
            .unwrap();

        context.surface.window().gl_swap_window();
        previous_frame_start = current_frame_start;

        glerror::assert_no_gl_error();
    }
}
