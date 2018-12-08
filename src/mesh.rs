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
    pub fn new (vertices: &[Vertex], indices: &[GLuint]) -> Mesh {
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
                (indices.len() * size_of::<GLuint>()) as GLsizeiptr,
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
                gl::UNSIGNED_INT,
                ptr::null()
                );
            gl::BindVertexArray(0);
        }
    }

    #[allow(dead_code)]
    pub fn cube () -> Mesh {
        use glm::vec3;
        let vector = |arr: &[i32]| vec3(arr[0] as f32, arr[1] as f32, arr[2] as f32);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for face in 0..6 {
            let direction = if face < 3 { -1 } else { 1 };
            let plane = face % 3;
            let normal = {
                let mut arr = [0; 3];
                arr[plane] = direction;
                vector(&arr)
            };
            let color = if direction > 0 { normal } else { normal + 1.0 };
            vertices.extend(
                [
                [0, 0],
                [0, 1],
                [1, 1],
                [1, 0],
                ].iter()
                .map(|[a, b]| {
                    let position = {
                        let mut corner = [(direction + 1) / 2, *a, *b];
                        corner.rotate_right(plane);
                        vector(&corner) - 0.5
                    };
                    Vertex { position, normal, color }
                })
                );
            indices.extend(
                {
                    let mut new = [
                        0, 1, 2,
                        2, 3, 0,
                    ];
                    if direction < 0 { new.reverse(); }
                    new
                }.iter().map(|i| face as GLuint * 4 + i)
                );
        };
        Mesh::new(&vertices, indices.as_slice())
    }
}
