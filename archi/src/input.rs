use glutin::event;

#[derive(Default, Debug)]
pub struct Movement {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl Movement {
    pub fn update(&mut self, event: &event::Event<()>) {
        if let event::Event::WindowEvent {
            event:
                event::WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            scancode, state, ..
                        },
                    ..
                },
            ..
        } = event
        {
            if let Some(direction) = match scancode {
                17 => Some(&mut self.forward),
                30 => Some(&mut self.left),
                31 => Some(&mut self.backward),
                32 => Some(&mut self.right),
                _ => None,
            } {
                *direction = match state {
                    event::ElementState::Pressed => true,
                    event::ElementState::Released => false,
                }
            }
        }
    }

    fn axis(positive: bool, negative: bool) -> f32 {
        match (positive, negative) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        }
    }

    pub fn x_axis(&self) -> f32 {
        Self::axis(self.right, self.left)
    }

    pub fn y_axis(&self) -> f32 {
        Self::axis(self.up, self.down)
    }

    pub fn z_axis(&self) -> f32 {
        Self::axis(self.backward, self.forward)
    }

    pub fn axes(&self) -> (f32, f32, f32) {
        (self.x_axis(), self.y_axis(), self.z_axis())
    }
}
