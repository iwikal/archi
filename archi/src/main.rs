#![feature(track_caller)]
#[windows_subsystem = "windows"]
extern crate nalgebra_glm as glm;

use glutin::event::{
    DeviceEvent, Event, KeyboardInput, VirtualKeyCode, WindowEvent,
};
use luminance::context::GraphicsContext;

mod shader;

mod camera;
mod context;
mod debug;
mod fft;
mod glerror;
mod grid;
mod input;
mod ocean;
mod skybox;

fn main() {
    eprintln!("running!");

    let event_loop = glutin::event_loop::EventLoop::new();
    let (mut context, mut surface) = context::Surface::new(&event_loop);

    glerror::debug_messages(glerror::GlDebugSeverity::Low);

    let mut back_buffer = context.back_buffer(surface.size()).unwrap();

    let [width, height] = surface.size();
    let mut camera = camera::Camera::new(width, height);

    let mut skybox = {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        std::thread::spawn(move || {
            let image = skybox::Skybox::load_image().unwrap_or_else(|e| {
                eprintln!("{}", e);
                Default::default()
            });
            tx.send(image).unwrap();
        });

        enum Lazy<T, F: FnMut(&mut context::Context) -> Option<T>> {
            Pending(F),
            Done(T),
        }

        impl<T, F: FnMut(&mut context::Context) -> Option<T>> Lazy<T, F> {
            fn value(
                &mut self,
                context: &mut context::Context,
            ) -> Option<&mut T> {
                match self {
                    Self::Done(t) => Some(t),
                    Self::Pending(f) => match f(context) {
                        Some(t) => {
                            *self = Self::Done(t);
                            self.value(context)
                        }
                        None => None,
                    },
                }
            }
        }

        Lazy::Pending(move |context| {
            rx.try_recv()
                .ok()
                .map(|image| skybox::Skybox::new(context, image))
        })
    };

    let mut ocean = ocean::Ocean::new(&mut context);

    let mut exposure = 0.2;

    let mut render_stuff = true;

    let start = std::time::Instant::now();
    let mut last_input_read = start;

    let mut input_state = input::InputState::default();

    let mut debugger = debug::Debugger::new(&mut context);

    surface.ctx.window().set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Poll;

        input_state.update(&event);

        match event {
            Event::NewEvents(..) => {
                surface.ctx.window().request_redraw();
            }
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta: (x, y) } => {
                    camera.mouse_moved(x as f32, y as f32);
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
                    exposure *= 2.0_f32.powf(y);
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(..) => {
                    let [width, height] = surface.size();
                    back_buffer = context.back_buffer([width, height]).unwrap();
                    camera.update_dimensions(width, height);
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            scancode: 18,
                            state: glutin::event::ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    render_stuff = !render_stuff;
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let now = std::time::Instant::now();
                let delta_t = now - last_input_read;
                camera.take_input(&input_state);
                let delta_f = delta_t.as_micros() as f32 / 1_000_000.0;
                camera.physics_tick(delta_f);
                last_input_read = now;
            }
            Event::RedrawRequested(..) => {
                let now = std::time::Instant::now();
                let t = (now - start).as_secs_f32();

                let mut skybox = skybox.value(&mut context);

                let mut pipeline_gate = context.new_pipeline_gate();

                let mut ocean_frame = None;
                if render_stuff {
                    ocean_frame = Some(ocean.simulate(&mut pipeline_gate, t));
                }

                use luminance::pipeline::PipelineState;

                pipeline_gate
                    .pipeline(
                        &back_buffer,
                        &PipelineState::new(),
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
                                    skybox.as_mut().map(|s| s.texture()),
                                    exposure,
                                );
                            }

                            for &pos in &[
                                glm::vec3(0.0, 0.0, 0.0),
                                glm::vec3(1.0, 0.0, 0.0),
                                glm::vec3(-1.0, 0.0, 0.0),
                                glm::vec3(0.0, 1.0, 0.0),
                                glm::vec3(0.0, -1.0, 0.0),
                                glm::vec3(0.0, 0.0, 1.0),
                                glm::vec3(0.0, 0.0, -1.0),
                            ] {
                                let model: glm::Mat4 = glm::identity();
                                let model = glm::translate(&model, &pos);
                                let model = glm::scale(
                                    &model,
                                    &glm::vec3(0.2, 0.2, 0.2),
                                );
                                let model = model * camera.orientation();

                                debugger.render(
                                    &pipeline,
                                    &mut shader_gate,
                                    view_projection,
                                    model,
                                    None,
                                );
                            }

                            if let Some(skybox) = skybox {
                                skybox.render(
                                    &pipeline,
                                    &mut shader_gate,
                                    view,
                                    projection,
                                    exposure,
                                );
                            }
                        },
                    )
                    .unwrap();

                surface.swap_buffers();

                glerror::print_gl_errors();
            }
            _ => {}
        }
    });
}
