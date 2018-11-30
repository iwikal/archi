use camera::Camera;
use shader::Shader;
use mesh::Mesh;

#[derive(Debug)]
pub struct Model {
    mesh: &'static Mesh,
    shader: Shader,
}

impl Model {
    pub fn new (mesh: &'static Mesh, shader: Shader) -> Model {
        Model {
            mesh,
            shader,
        }
    }
}

pub trait Renderable {
    fn render(&self, camera: &Camera) -> ();
}

impl Renderable for Model {
    fn render(&self, camera: &Camera) {
        let view = camera.view();
        let projection = camera.projection();
        let projection_view = projection * view;

        let pv_location = self.shader.get_location("projection_view");

        self.shader.activate();
        unsafe {
            gl::UniformMatrix4fv(pv_location,
                                 1,
                                 gl::FALSE,
                                 &(projection_view[0][0]));
            self.mesh.draw();
            gl::UseProgram(0);
        }
    }
}
