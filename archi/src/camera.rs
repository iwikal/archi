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
    pub fn new(width: u32, height: u32) -> Self {
        let mut cam = Self {
            position: glm::vec3(0.0, 1.0, 0.0),
            velocity: glm::zero(),
            acceleration: glm::zero(),
            pitch: 0.,
            yaw: 0.,
            orientation: glm::identity(),
            projection: glm::identity(),
        };

        cam.update_dimensions(width, height);
        cam
    }

    pub fn update_dimensions(&mut self, width: u32, height: u32) {
        let aspect = width as f32 / height as f32;
        let fov = 1.1;
        let near = 0.1;
        self.projection = glm::infinite_perspective_rh_no(aspect, fov, near);
    }

    fn mouse_moved(&mut self, x: f64, y: f64) {
        let scale = 1.0 / 128.0;
        self.yaw -= (x * scale) as f32;
        self.pitch -= (y * scale) as f32;

        self.orientation =
            glm::Mat4::from_euler_angles(self.pitch, self.yaw, 0.0);
    }

    pub fn take_input(&mut self, input: &crate::input::Input) {
        let (x, y, z) = input.movement().axes();

        let (x, z) = (
            // Map x and z from square to disc
            x * (1.0 - z * z / 2.0).sqrt(),
            z * (1.0 - x * x / 2.0).sqrt(),
        );

        let mut move_vector = glm::vec3(x, y, z);

        let length = glm::length(&move_vector);
        let length = if length > 1.0 { length } else { 1.0 };
        move_vector /= length;

        self.acceleration = SPEED * glm::rotate_y_vec3(&move_vector, self.yaw);

        let (x, y) = input.mouse().delta_axes();
        self.mouse_moved(x as _, y as _);
    }

    pub fn physics_tick(&mut self, delta_t: f32) {
        let friction: f32 = 100.0;
        self.velocity *= (1. / friction).powf(delta_t);
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
        glm::translate(&glm::transpose(&self.orientation), &-self.position)
    }
}
