extern crate glm;

use std::ptr;
use std::ffi::CString;
use gl::types::*;
use shader::Shader;
use camera::Camera;

pub struct Vertex {
    pub position: glm::Vec3
}

pub struct Mesh {
    vao: GLuint,
    elements: GLsizei,
    program: Shader,
}

pub fn new (vertices: &[Vertex],
            indices: &[GLushort],
            program: Shader) -> Mesh {
    let mut vao = 0;
    let mut vbo = 0;
    let mut ebo = 0;

    unsafe {
        use std::mem::{ size_of, transmute };
        gl::CreateVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::CreateBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * size_of::<Vertex>()) as GLsizeiptr,
            transmute(&vertices[0]),
            gl::STATIC_DRAW);

        gl::CreateBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * size_of::<GLushort>()) as GLsizeiptr,
            transmute(&indices[0]),
            gl::STATIC_DRAW);

        program.activate();

        // Specify the layout of the vertex data
        let pos_attr = gl::GetAttribLocation(
            program.name,
            CString::new("position").unwrap().as_ptr()
            ) as GLuint;
        gl::EnableVertexAttribArray(pos_attr);
        gl::VertexAttribPointer(
            pos_attr,
            3,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            0,
            ptr::null(),
        );

        gl::UseProgram(0);
        gl::BindVertexArray(0);
    }

    Mesh {
        vao,
        elements: indices.len() as GLsizei,
        program,
    }
}

pub trait Renderable {
    fn render(&self, camera: &Camera) -> ();
}

impl Renderable for Mesh {
    fn render(&self, camera: &Camera) {
        let view = camera.view();
        let projection = camera.projection();
        let projection_view = projection * view;

        let pv_location = self.program.get_location("projection_view");

        self.program.activate();
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UniformMatrix4fv(pv_location,
                                 1,
                                 gl::FALSE,
                                 &(projection_view[0][0]));
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.elements,
                gl::UNSIGNED_SHORT,
                ptr::null()
                );
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }
}
