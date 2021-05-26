use glutin::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta,
    WindowEvent,
};

#[derive(Default, Debug)]
pub struct Mouse {
    delta_x: f64,
    delta_y: f64,
    scroll: f32,
}

impl Mouse {
    fn update(&mut self, event: &DeviceEvent) {
        match *event {
            DeviceEvent::MouseMotion { delta: (x, y) } => {
                self.delta_x = x;
                self.delta_y = y;
            }
            DeviceEvent::MouseWheel { delta } => {
                self.scroll = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => y / 10.0,
                    MouseScrollDelta::PixelDelta(pos) => pos.x as f32 / 100.0,
                };
            }
            _ => (),
        }
    }

    pub fn delta_axes(&self) -> (f64, f64) {
        (self.delta_x, self.delta_y)
    }

    pub fn scroll(&self) -> f32 {
        self.scroll
    }
}

#[derive(Default, Debug)]
pub struct Movement {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    mouse: Mouse,
}

impl Movement {
    pub fn update(&mut self, input: &KeyboardInput) {
        let KeyboardInput {
            scancode, state, ..
        } = input;

        if let Some(direction) = match scancode {
            17 => Some(&mut self.forward),
            30 => Some(&mut self.left),
            31 => Some(&mut self.backward),
            32 => Some(&mut self.right),
            57 => Some(&mut self.up),
            42 => Some(&mut self.down),
            _ => None,
        } {
            *direction = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
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

#[derive(Default, Debug)]
pub struct Input {
    movement: Movement,
    mouse: Mouse,
}

impl Input {
    pub fn movement(&self) -> &Movement {
        &self.movement
    }

    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }

    pub fn update(&mut self, event: &Event<()>) {
        match event {
            Event::NewEvents(..) => self.mouse = Default::default(),
            Event::DeviceEvent { event, .. } => self.mouse.update(event),
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => self.movement.update(input),
            _ => (),
        }
    }
}
