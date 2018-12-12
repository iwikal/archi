use gl::types::*;
use glm::{ Vec3 };
use shader::Shader;
use camera::Camera;
use model::Model;
use mesh::Mesh;
use glerror;

pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
}

pub struct PointLight {
    pub radius: f32,
    pub position: Vec3,
    pub color: Vec3,
}

struct Framebuffer {
    name: GLuint,
    buffers: Vec<GLuint>,
}

impl Framebuffer {
    pub fn new (width: i32, height: i32, formats: &[GLenum]) -> Self {
        unsafe {
            let mut name = 0;
            gl::GenFramebuffers(1, &mut name);
            gl::BindFramebuffer(gl::FRAMEBUFFER, name);

            let buffers = {
                formats.iter()
                    .enumerate()
                    .map(|(i, &format)| {
                        let mut buffer = 0;
                        gl::GenTextures(1, &mut buffer);
                        gl::BindTexture(gl::TEXTURE_2D, buffer);
                        gl::TexImage2D(gl::TEXTURE_2D,
                                       0,
                                       format as GLint,
                                       width,
                                       height,
                                       0,
                                       gl::RGB,
                                       gl::UNSIGNED_BYTE,
                                       std::ptr::null());
                        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

                        gl::FramebufferTexture(gl::FRAMEBUFFER,
                                               gl::COLOR_ATTACHMENT0 + i as GLuint,
                                               buffer,
                                               0);
                        buffer
                    })
                .collect::<Vec<GLuint>>()
            };

            let mut depth_buffer = 0;
            gl::GenTextures(1, &mut depth_buffer);
            gl::BindTexture(gl::TEXTURE_2D, depth_buffer);
            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           gl::DEPTH_COMPONENT as i32,
                           width,
                           height,
                           0,
                           gl::DEPTH_COMPONENT,
                           gl::UNSIGNED_BYTE,
                           std::ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            gl::FramebufferTexture(gl::FRAMEBUFFER,
                                   gl::DEPTH_ATTACHMENT,
                                   depth_buffer,
                                   0);

            let attachments = [
                gl::COLOR_ATTACHMENT0,
                gl::COLOR_ATTACHMENT1,
                gl::COLOR_ATTACHMENT2,
            ];
            gl::DrawBuffers(3, &attachments[0]);

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("framebuffer not complete");
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            assert_no_gl_error!();

            Self {
                name,
                buffers,
            }
        }
    }

    pub fn bind (&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.name);
        }
    }

    pub fn unbind () {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}

static VS_SRC: &'static str = include_str!("shaders/basicShader.vert");
static FS_SRC: &'static str = include_str!("shaders/basicShader.frag");

static AMBIENT_VERT: &'static str = include_str!("shaders/ambient.light.vert");
static AMBIENT_FRAG: &'static str = include_str!("shaders/ambient.light.frag");

static POINT_VERT: &'static str = include_str!("shaders/point.light.vert");
static POINT_FRAG: &'static str = include_str!("shaders/point.light.frag");

static DIR_VERT: &'static str = include_str!("shaders/ambient.light.vert");
static DIR_FRAG: &'static str = include_str!("shaders/directional.light.frag");

pub struct Renderer {
    framebuffer: Framebuffer,
    model_shader: Shader,
    ambient_shader: Shader,
    directional_shader: Shader,
    quad_mesh: Mesh,
    point_shader: Shader,
    point_mesh: Mesh,
}

impl Renderer {
    pub fn new (width: i32, height: i32) -> Self {
        Self {
            framebuffer: {
                let formats = [
                    gl::RGB,
                    gl::RGBA16F,
                    gl::RGB16F,
                ];

                Framebuffer::new(width, height, &formats)
            },
            model_shader: Shader::from_vert_frag(VS_SRC, FS_SRC),
            ambient_shader: Shader::from_vert_frag(AMBIENT_VERT, AMBIENT_FRAG),
            directional_shader: Shader::from_vert_frag(DIR_VERT, DIR_FRAG),
            quad_mesh: Mesh::ambient_light(),
            point_shader: Shader::from_vert_frag(POINT_VERT, POINT_FRAG),
            point_mesh: Mesh::point_light(1.0),
        }
    }

    pub fn render (
        &self,
        camera: &Camera,
        models: &[Model],
        ambient: glm::Vec3,
        directional_lights: &[DirectionalLight],
        point_lights: &[PointLight],
        ) {
        let projection = camera.projection();
        let view = camera.view();
        let view_projection = projection * view;

        self.framebuffer.bind();
        self.model_shader.activate();
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        };
        for m in models.iter() {
            m.render(&camera);
        }
        Framebuffer::unbind();

        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);

            gl::Clear(gl::COLOR_BUFFER_BIT);
            let buffers = &(self.framebuffer.buffers);
            gl::BindTextures(0, buffers.len() as i32, &buffers[0]);
        }
        self.ambient_shader.activate();
        unsafe {
            gl::Uniform3fv(1, 1, &ambient[0]);
        }
        self.quad_mesh.draw();
        self.directional_shader.activate();
        for light in directional_lights.iter() {
            unsafe {
                gl::Uniform3fv(1, 1, &light.direction[0]);
                gl::Uniform3fv(2, 1, &light.color[0]);
            }
            self.quad_mesh.draw();
        }

        self.point_shader.activate();
        for light in point_lights.iter() {
            let &PointLight { position, color, radius } = light;
            let model = num::one();
            let model = glm::ext::translate(&model, position);
            let model = glm::ext::scale(&model, glm::vec3(radius, radius, radius));
            let mvp = view_projection * model;

            unsafe {
                gl::UniformMatrix4fv(1,
                                     1,
                                     gl::FALSE,
                                     &(mvp[0][0]));

                gl::Uniform3fv(2, 1, &position[0]);
                gl::Uniform3fv(3, 1, &color[0]);
                gl::Uniform1f(4, radius);
            }
            self.point_mesh.draw();
        }
        Shader::deactivate();
    }
}
