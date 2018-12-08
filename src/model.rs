use camera::Camera;
use shader::Shader;
use mesh::{ Vertex, Mesh };
use glm::Mat4;
use std::path::Path;
use gl::types::*;

#[derive(Debug, Copy, Clone)]
pub struct Model {
    mesh: &'static Mesh,
    shader: &'static Shader,
    pub transform: Mat4,
}

static MVP_LOCATION: GLint = 3;
static MODEL_LOCATION: GLint = 4;

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

        self.shader.activate();
        unsafe {
            gl::UniformMatrix4fv(MVP_LOCATION,
                                 1,
                                 gl::FALSE,
                                 &(mvp_matrix[0][0]));
            gl::UniformMatrix4fv(MODEL_LOCATION,
                                 1,
                                 gl::FALSE,
                                 &(self.transform[0][0]));
            self.mesh.draw();
            gl::UseProgram(0);
        }
    }
}

pub fn from_obj (
    path: &str,
    scale: f32,
    reverse_winding: bool,
    ) -> Vec<Mesh> {
    use tobj::*;
    use mesh;
    let (models, _materials) = load_obj(Path::new(path)).unwrap();
    models.iter()
        .map(|Model { mesh, .. }| {
            let positions = mesh.positions.as_slice()
                .chunks_exact(3)
                .map(|chunk| {
                    glm::vec3(chunk[0], chunk[1], chunk[2])
                });
            let normals = mesh.normals.as_slice()
                .chunks_exact(3)
                .map(|chunk| {
                    if reverse_winding {
                        glm::vec3(chunk[0], chunk[1], chunk[2])
                    } else {
                        glm::vec3(chunk[2], chunk[1], chunk[0])
                    }
                });
            let vertices: Vec<Vertex> = positions.zip(normals)
                .map(|(position, normal)| Vertex {
                    position: position * scale,
                    normal,
                    color: glm::vec3(1.0, 1.0, 1.0),
                })
            .collect();
            mesh::Mesh::new(&vertices, mesh.indices.as_slice())
        })
        .collect()
}
