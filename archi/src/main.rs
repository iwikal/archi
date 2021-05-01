#[windows_subsystem = "windows"]
extern crate nalgebra_glm as glm;

use anyhow::Context as _;
use glutin::{
    event::{DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use luminance_front::context::GraphicsContext;

mod shader;

mod camera;
mod context;
mod debug;
mod fft;
mod grid;
mod input;
mod ocean;
mod skybox;

fn start_loading() -> std::time::Instant {
    eprintln!("loading...");
    std::time::Instant::now()
}

fn finish_loading(start: std::time::Instant) {
    let loading_duration = start.elapsed();
    eprintln!("finished loading in {:.2}s", loading_duration.as_secs_f32());
}

fn main() -> anyhow::Result<()> {
    let loading_start = start_loading();

    let event_loop = EventLoop::new();
    let (mut context, mut surface) = context::Surface::new(&event_loop);

    debug::glerr::debug_messages(debug::glerr::GlDebugSeverity::Low);

    let [width, height] = surface.size();

    let start = std::time::Instant::now();
    let mut last_input_read = start;

    surface.ctx.window().set_visible(true);

    let mut state = AppState {
        back_buffer: context.back_buffer(surface.size())?,
        camera: camera::Camera::new(width, height),
        movement: input::Movement::default(),
        skybox: skybox::Skybox::new(&mut context)?,
        ocean: ocean::Ocean::new(&mut context)?,
        exposure: 0.2,
        render_stuff: true,
    };

    let mut on_event = move |event: Event<()>,
                             control_flow: &mut ControlFlow|
          -> anyhow::Result<()> {
        *control_flow = input(&event, &mut state);

        match event {
            Event::NewEvents(..) => {
                surface.ctx.window().request_redraw();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(..) => {
                    let [width, height] = surface.size();
                    state.back_buffer = context.back_buffer([width, height])?;
                    state.camera.update_dimensions(width, height);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let now = std::time::Instant::now();
                let delta_t = now - last_input_read;
                state.camera.take_input(&state.movement);
                let delta_f = delta_t.as_micros() as f32 / 1_000_000.0;
                state.camera.physics_tick(delta_f);
                last_input_read = now;
            }
            Event::RedrawRequested(..) => {
                let now = std::time::Instant::now();
                let t = (now - start).as_secs_f32();

                draw(t, &mut context, &mut state)
                    .context("Failed to render")?;

                surface.swap_buffers();

                debug::glerr::print_gl_errors();
            }
            _ => {}
        }

        Ok(())
    };

    finish_loading(loading_start);

    event_loop.run(move |event, _, control_flow| {
        match on_event(event, control_flow).context("Failed to process event") {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {:?}", e);

                *control_flow = ControlFlow::Exit;
            }
        }
    });
}

struct AppState {
    back_buffer: context::BackBuffer,
    camera: camera::Camera,
    exposure: f32,
    movement: input::Movement,
    ocean: ocean::Ocean,
    render_stuff: bool,
    skybox: skybox::Skybox,
}

fn draw(
    t: f32,
    context: &mut context::Context,
    state: &mut AppState,
) -> anyhow::Result<()> {
    let AppState {
        back_buffer,
        camera,
        exposure,
        ocean,
        render_stuff,
        skybox,
        ..
    } = state;

    let mut pipeline_gate = context.new_pipeline_gate();

    let ocean_frame = match render_stuff {
        true => Some(ocean.simulate(&mut pipeline_gate, t)?),
        false => None,
    };

    use luminance_front::pipeline::PipelineState;

    pipeline_gate
        .pipeline(
            &back_buffer,
            &PipelineState::new().enable_srgb(true),
            |mut pipeline, mut shader_gate| {
                let view = camera.view();
                let projection = camera.projection();

                let view_projection = projection * view;

                if let Some(mut ocean_frame) = ocean_frame {
                    ocean_frame.render(
                        &pipeline,
                        &mut shader_gate,
                        view_projection,
                        camera.position(),
                        Some(&mut skybox.sky_texture),
                        *exposure,
                    )?;
                }

                skybox.render(
                    &mut pipeline,
                    &mut shader_gate,
                    view,
                    projection,
                    *exposure,
                )
            },
        )
        .into_result()?;

    Ok(())
}

fn input(event: &Event<()>, state: &mut AppState) -> ControlFlow {
    state.movement.update(&event);

    match event {
        Event::DeviceEvent { event, .. } => match event {
            &DeviceEvent::MouseMotion { delta: (x, y) } => {
                state.camera.mouse_moved(x, y);
            }
            DeviceEvent::MouseWheel { delta, .. } => {
                let y = match delta {
                    glutin::event::MouseScrollDelta::LineDelta(_x, y) => {
                        y / 10.0
                    }
                    glutin::event::MouseScrollDelta::PixelDelta(pos) => {
                        pos.x as f32 / 100.0
                    }
                };
                state.exposure *= 2.0_f32.powf(y);
            }
            _ => {}
        },
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: glutin::event::ElementState::Released,
                            virtual_keycode,
                            scancode,
                            ..
                        },
                    ..
                },
            ..
        } => match (virtual_keycode, scancode) {
            (_, 18) => {
                state.render_stuff = !state.render_stuff;
            }
            (Some(VirtualKeyCode::Escape), _) => {
                return ControlFlow::Exit;
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            state.camera.take_input(&state.movement);
        }
        _ => {}
    }

    ControlFlow::Poll
}
