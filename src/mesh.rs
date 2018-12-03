extern crate glm;

use std::ptr;
use gl::types::*;

#[derive(Debug)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub color: glm::Vec3,
}

#[derive(Debug)]
pub struct Mesh {
    vao: GLuint,
    elements: GLsizei,
}

static POSITION_LOCATION: GLuint = 0;
static NORMAL_LOCATION: GLuint = 1;
static COLOR_LOCATION: GLuint = 2;

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
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

            gl::EnableVertexAttribArray(POSITION_LOCATION);
            gl::VertexAttribPointer(
                POSITION_LOCATION,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, position) as *const _);

            gl::EnableVertexAttribArray(NORMAL_LOCATION);
            gl::VertexAttribPointer(
                NORMAL_LOCATION,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, normal) as *const _);

            gl::EnableVertexAttribArray(COLOR_LOCATION);
            gl::VertexAttribPointer(
                COLOR_LOCATION,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, color) as *const _);

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
        let vector = |arr: &[i32]| vec3(arr[0] as f32, arr[1] as f32, arr[2] as f32);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for plane in 0..3 {
            for direction in &[-1, 1] {
                let normal = {
                    let mut arr = [0; 3];
                    arr[plane] = *direction;
                    vector(&arr)
                };
                let color = if direction > &0 { normal } else { normal + 1.0 };
                let l = vertices.len() as GLushort;
                for a in &[[0, 0], [0, 1], [1, 1], [1, 0]] {
                    let position = {
                        let mut slice = [(direction + 1) / 2, a[0], a[1]];
                        slice.rotate_right(plane);
                        vector(&slice) - 0.5
                    };
                    vertices.push(Vertex { position, normal, color });
                };
                indices.extend(&{
                    let mut new = [
                        l + 0, l + 1, l + 2,
                        l + 2, l + 3, l + 0,
                    ];
                    if direction < &0 { new.reverse(); }
                    new
                });
            };
        };
        Mesh::new(&vertices, indices.as_slice())
    }
}
