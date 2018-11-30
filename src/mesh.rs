extern crate glm;

use std::ptr;
use gl::types::*;

pub struct Vertex {
    pub position: glm::Vec3
}

#[derive(Debug)]
pub struct Mesh {
    vao: GLuint,
    elements: GLsizei,
}

impl Mesh {
    pub fn new (vertices: &[Vertex], indices: &[GLushort]) -> Mesh {
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

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
                );

            gl::BindVertexArray(0);
        }

        Mesh {
            vao,
            elements: indices.len() as GLsizei,
        }
    }

    pub fn draw (&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.elements,
                gl::UNSIGNED_SHORT,
                ptr::null()
                );
            gl::BindVertexArray(0);
        }
    }

    pub fn cube () -> Mesh {
        use glm::vec3;
        let vertices = [
            Vertex { position: vec3(-0.5, -0.5, -0.5) },
            Vertex { position: vec3( 0.5, -0.5, -0.5) },
            Vertex { position: vec3( 0.5,  0.5, -0.5) },
            Vertex { position: vec3(-0.5,  0.5, -0.5) },
            Vertex { position: vec3(-0.5,  0.5,  0.5) },
            Vertex { position: vec3( 0.5,  0.5,  0.5) },
            Vertex { position: vec3( 0.5, -0.5,  0.5) },
            Vertex { position: vec3(-0.5, -0.5,  0.5) },
        ];
        let indices = [
            0, 1, 2,
            0, 2, 3,
            0, 3, 4,
            0, 4, 7,
            0, 7, 6,
            0, 6, 1,
            2, 1, 5,
            3, 2, 5,
            4, 3, 5,
            7, 4, 5,
            6, 7, 5,
            1, 6, 5,
        ];
        Mesh::new(&vertices, &indices)
    }
}
