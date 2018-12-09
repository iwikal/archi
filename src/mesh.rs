extern crate glm;

use std::ptr;
use gl::types::*;

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}

pub trait Vertex {
    unsafe fn init_attrib_pointers ();
}

#[derive(Debug)]
pub struct ModelVertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub color: glm::Vec3,
}

impl Vertex for ModelVertex {
    unsafe fn init_attrib_pointers () {
        use std::mem::{ size_of };
        gl::EnableVertexAttribArray(POSITION_LOCATION);
        gl::VertexAttribPointer(
            POSITION_LOCATION,
            3,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            size_of::<Self>() as GLsizei,
            offset_of!(Self, position) as *const _);

        gl::EnableVertexAttribArray(NORMAL_LOCATION);
        gl::VertexAttribPointer(
            NORMAL_LOCATION,
            3,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            size_of::<Self>() as GLsizei,
            offset_of!(Self, normal) as *const _);

        gl::EnableVertexAttribArray(COLOR_LOCATION);
        gl::VertexAttribPointer(
            COLOR_LOCATION,
            3,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            size_of::<Self>() as GLsizei,
            offset_of!(Self, color) as *const _);
    }
}

#[derive(Debug)]
pub struct Mesh {
    vao: GLuint,
    elements: GLsizei,
}

static POSITION_LOCATION: GLuint = 0;
static NORMAL_LOCATION: GLuint = 1;
static COLOR_LOCATION: GLuint = 2;

struct LightVertex {
    position: glm::Vec3,
}

impl Vertex for LightVertex {
    unsafe fn init_attrib_pointers () {
        use std::mem::{ size_of };
        gl::EnableVertexAttribArray(POSITION_LOCATION);
        gl::VertexAttribPointer(
            POSITION_LOCATION,
            3,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            size_of::<Self>() as GLsizei,
            offset_of!(Self, position) as *const _);
    }
}

impl Mesh {
    pub fn ambient_light () -> Mesh {
        let (positions, indices) = Mesh::quad();

        let vertices: Vec<LightVertex> = positions.into_iter()
            .map(|position| {
                LightVertex {
                    position,
                }
            }).collect();

        Mesh::new(&vertices, &indices)
    }

    pub fn new<T> (vertices: &[T], indices: &[GLuint]) -> Mesh
        where T: Vertex
    {
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
                (vertices.len() * size_of::<T>()) as GLsizeiptr,
                transmute(&vertices[0]),
                gl::STATIC_DRAW);

            gl::CreateBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * size_of::<GLuint>()) as GLsizeiptr,
                transmute(&indices[0]),
                gl::STATIC_DRAW);

            T::init_attrib_pointers();
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
    pub fn cube () -> (Vec<glm::Vec3>, Vec<GLuint>) {
        use glm::vec3;
        let vector = |arr: &[i32]| vec3(arr[0] as f32, arr[1] as f32, arr[2] as f32);
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for face in 0..6 {
            let direction = if face < 3 { -1 } else { 1 };
            let plane = face % 3;
            positions.extend(
                [
                [0, 0],
                [1, 0],
                [1, 1],
                [0, 1],
                ].iter()
                .map(|[a, b]| {
                    let position = {
                        let mut corner = [(direction + 1) / 2, *a, *b];
                        corner.rotate_right(plane);
                        vector(&corner) - 0.5
                    };
                    position
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
        (positions, indices)
    }

    #[allow(dead_code)]
    pub fn quad () -> (Vec<glm::Vec3>, Vec<GLuint>) {
        use glm::vec3;
        let positions = vec![
            vec3(-1., -1., 0.),
            vec3( 1., -1., 0.),
            vec3( 1.,  1., 0.),
            vec3(-1.,  1., 0.),
        ];
        let indices = vec![
            0, 1, 2,
            2, 3, 0
        ];
        (positions, indices)
    }
}
