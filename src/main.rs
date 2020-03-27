extern crate nalgebra_glm as glm;
use luminance::{context::GraphicsContext, framebuffer::Framebuffer};

mod shader;

mod camera;
mod context;
mod debug;
mod fft;
mod glerror;
mod grid;
mod ocean;
mod skybox;
mod terrain;

fn main() {
    let context = &mut context::Context::new(800, 600);

    let (width, height) = context.window.size();

    let mut event_pump = context.sdl.event_pump().unwrap();
    let mut back_buffer = Framebuffer::back_buffer(context, [width, height]);

    let mut camera =
        camera::Camera::persp(width as f32 / height as f32, 0.9, 0.1, 1000.0);

    let skybox = skybox::Skybox::new(context, "/home/iwikal/poods");
    let mut ocean = ocean::Ocean::new(context);
    let terrain = terrain::Terrain::new(context, "assets/heightmap.png");

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
                    if let Some(Escape) = scancode {
                        break 'game_loop;
                    }
                }
                Event::MouseMotion { xrel, yrel, .. } => {
                    camera.mouse_moved(xrel as f32, yrel as f32);
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

        if let Some([width, height]) = resize {
            let size = [width as u32, height as u32];
            back_buffer = Framebuffer::back_buffer(context, size);
        }

        camera.take_input(&event_pump);
        let delta_f = delta_t.as_micros() as f32 / 1_000_000.0;
        camera.physics_tick(delta_f);

        let mut builder = context.pipeline_builder();

        let duration = current_frame_start - start;
        let f_time = duration.as_secs() as f32
            + duration.subsec_nanos() as f32 / 1_000_000_000.0;

        let ocean_frame = ocean.simulate(&mut builder, f_time);

        let pipeline_state = luminance::pipeline::PipelineState::new()
            .set_clear_color([0.1, 0.2, 0.3, 1.0]);
        builder.pipeline(
            &back_buffer,
            &pipeline_state,
            |pipeline, mut shader_gate| {
                let view = camera.view();
                let projection = camera.projection();

                let view_projection = projection * view;

                ocean_frame.render(
                    &pipeline,
                    &mut shader_gate,
                    view_projection,
                );

                terrain.render(&pipeline, &mut shader_gate, view_projection);

                skybox.render(&pipeline, &mut shader_gate, view, projection);
            },
        );

        context.window.gl_swap_window();
        previous_frame_start = current_frame_start;

        glerror::assert_no_gl_error();
    }
}
