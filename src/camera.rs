extern crate sdl2;
extern crate glm;
extern crate num;

pub struct Camera {
    position: glm::Vec3,
    pitch: f32,
    yaw: f32,
    projection: glm::Mat4
}

impl Camera {
    pub fn new () -> Camera {
        Camera {
            position: glm::vec3(0., 0., -1.),
            pitch: 0.,
            yaw: 0.,
            projection: glm::ext::perspective(1.0, 4.0 / 3.0, 0.1, 100.0),
        }
    }

    pub fn take_input (&mut self, pump: &sdl2::EventPump) {
        {
            let scale = 5.;
            let state = pump.relative_mouse_state();
            self.yaw += (state.x() as f32 / 800.) * scale;
            self.pitch += (state.y() as f32 / 600.) * scale;
        }
        {
            let state = pump.keyboard_state();

            use sdl2::keyboard::*;

            let mut move_vector = glm::vec3(0., 0., 0.);

            let is_pressed = |name| {
                let key = Scancode::from_name(name).unwrap();
                state.is_scancode_pressed(key)
            };

            if is_pressed("W") { move_vector = move_vector + glm::vec3(0., 0., 1.); }
            if is_pressed("A") { move_vector = move_vector + glm::vec3(1., 0., 0.); }
            if is_pressed("S") { move_vector = move_vector + glm::vec3(0., 0., -1.); }
            if is_pressed("D") { move_vector = move_vector + glm::vec3(-1., 0., 0.); }

            let move_vector = move_vector * 0.125;

            let move_vector = move_vector * glm::max(glm::length(move_vector), 1.0);
            let move_vector = move_vector.extend(1.0);
            let move_vector = glm::inverse(&self.orientation()) * move_vector;
            let move_vector = move_vector.truncate(3);
            self.position = self.position + move_vector;
        }
    }

    fn orientation (&self) -> glm::Mat4 {
        let ori = glm::ext::rotate(&num::one(), self.pitch, glm::vec3(1., 0., 0.));
        let ori = glm::ext::rotate(&ori, self.yaw, glm::vec3(0., 1., 0.));
        ori
    }

    pub fn projection (&self) -> glm::Mat4 { self.projection }

    pub fn view (&self) -> glm::Mat4 {
        let view: glm::Mat4 = num::one();
        let view = view * self.orientation();
        let view = glm::ext::translate(&view, self.position);

        view
    }
}
