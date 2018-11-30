use camera::Camera;
use shader::Shader;
use mesh::Mesh;
use glm::Mat4;

#[derive(Debug)]
pub struct Model {
    mesh: &'static Mesh,
    shader: &'static Shader,
    pub transform: Mat4,
}

impl Model {
    pub fn new (
        mesh: &'static Mesh,
        shader: &'static Shader,
        transform: Mat4,
        ) -> Model {
        Model {
            mesh,
            shader,
            transform,
        }
    }

    pub fn render(&self, camera: &Camera) {
        let view = camera.view();
        let projection = camera.projection();
        let mvp_matrix = projection * view * self.transform;

        let mvp_location = self.shader.get_location("model_view_projection");

        self.shader.activate();
        unsafe {
            gl::UniformMatrix4fv(mvp_location,
                                 1,
                                 gl::FALSE,
                                 &(mvp_matrix[0][0]));
            self.mesh.draw();
            gl::UseProgram(0);
        }
    }
}
