#[derive(Debug)]
pub struct Camera {
    position: glm::Vec3,
    velocity: glm::Vec3,
    acceleration: glm::Vec3,
    pitch: f32,
    yaw: f32,
    orientation: glm::Mat4,
    projection: glm::Mat4,
}

const SPEED: f32 = 50.0;

impl Camera {
    fn new(projection: glm::Mat4) -> Self {
        Self {
            position: glm::zero(),
            velocity: glm::zero(),
            acceleration: glm::zero(),
            pitch: 0.,
            yaw: 0.,
            orientation: glm::identity(),
            projection,
        }
    }

    pub fn persp(aspect: f32, fov: f32, near: f32, far: f32) -> Self {
        let projection = glm::perspective_rh(aspect, fov, near, far);
        Self::new(projection)
    }

    pub fn take_input(&mut self, pump: &sdl2::EventPump) {
        {
            let scale = 1.0 / 128.0;
            let state = pump.relative_mouse_state();
            self.yaw += (state.x() as f32) * scale;
            self.pitch += (state.y() as f32) * scale;

            let ori = glm::rotate(
                &glm::identity(),
                self.pitch,
                &glm::vec3(1., 0., 0.),
            );
            let ori = glm::rotate(&ori, self.yaw, &glm::vec3(0., 1., 0.));
            self.orientation = ori;
        }
        {
            let state = pump.keyboard_state();

            use sdl2::keyboard::*;

            let mut move_vector = glm::vec3(0., 0., 0.);

            let is_pressed = |name| {
                let key = Scancode::from_name(name).unwrap();
                state.is_scancode_pressed(key)
            };

            if is_pressed("W") {
                move_vector += glm::vec3(0., 0., -1.);
            }
            if is_pressed("A") {
                move_vector += glm::vec3(-1., 0., 0.);
            }
            if is_pressed("S") {
                move_vector += glm::vec3(0., 0., 1.);
            }
            if is_pressed("D") {
                move_vector += glm::vec3(1., 0., 0.);
            }
            if is_pressed("Space") {
                move_vector += glm::vec3(0., 1., 0.);
            }
            if is_pressed("Left Shift") {
                move_vector += glm::vec3(0., -1., 0.);
            }
            let length = glm::length(&move_vector);
            let length = if length > 1.0 { length } else { 1.0 };
            move_vector /= length;

            self.acceleration = {
                let x = move_vector.x;
                let y = move_vector.y;
                let z = move_vector.z;
                let sin = self.yaw.sin();
                let cos = self.yaw.cos();
                glm::vec3(x * cos + z * -sin, y, z * cos + x * sin)
            };

            self.acceleration *= SPEED;
        }
    }

    pub fn physics_tick(&mut self, delta_t: f32) {
        self.velocity *= 0.99;
        self.velocity += self.acceleration * delta_t;
        self.position += self.velocity * delta_t;
        self.acceleration = glm::zero();
    }

    pub fn projection(&self) -> glm::Mat4 {
        self.projection
    }

    pub fn view(&self) -> glm::Mat4 {
        glm::translate(&self.orientation, &-self.position)
    }
}
