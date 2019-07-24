use gl::types::*;
use specs::{Component, VecStorage};

pub struct Material {
    pub diffuse_texture: GLuint,
    pub normal_texture: GLuint,
}

impl Component for Material {
    type Storage = VecStorage<Self>;
}
