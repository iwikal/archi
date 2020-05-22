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
            position: glm::vec3(0.0, 1.0, 0.0),
            velocity: glm::zero(),
            acceleration: glm::zero(),
            pitch: 0.,
            yaw: 0.,
            orientation: glm::identity(),
            projection,
        }
    }

    pub fn persp(aspect: f32, fov: f32, near: f32) -> Self {
        let projection = glm::infinite_perspective_rh_no(aspect, fov, near);
        Self::new(projection)
    }

    pub fn mouse_moved(&mut self, x: f32, y: f32) {
        let scale = 1.0 / 128.0;
        self.yaw += x * scale;
        self.pitch += y * scale;

        let ori =
            glm::rotate(&glm::identity(), self.pitch, &glm::vec3(1., 0., 0.));
        let ori = glm::rotate(&ori, self.yaw, &glm::vec3(0., 1., 0.));
        self.orientation = ori;
    }

    pub fn take_input(&mut self, pump: &sdl2::EventPump) {
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

    pub fn physics_tick(&mut self, delta_t: f32) {
        self.velocity *= 0.99;
        self.velocity += self.acceleration * delta_t;
        self.position += self.velocity * delta_t;
        self.acceleration = glm::zero();
    }

    pub fn projection(&self) -> glm::Mat4 {
        self.projection
    }

    pub fn position(&self) -> glm::Vec3 {
        self.position
    }

    pub fn view(&self) -> glm::Mat4 {
        glm::translate(&self.orientation, &-self.position)
    }
}
