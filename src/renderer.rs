use camera::Camera;
use gl::types::*;
use glerror;
use glm::Vec3;
use mesh::Mesh;
use model::Model;
use rand::prelude::*;
use shader::Shader;

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
    pub fn new(width: i32, height: i32, formats: &[GLenum]) -> Self {
        unsafe {
            let mut name = 0;
            gl::GenFramebuffers(1, &mut name);
            gl::BindFramebuffer(gl::FRAMEBUFFER, name);

            let buffers = {
                formats
                    .iter()
                    .enumerate()
                    .map(|(i, &format)| {
                        let mut buffer = 0;
                        gl::GenTextures(1, &mut buffer);
                        gl::BindTexture(gl::TEXTURE_2D, buffer);
                        gl::TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            format as GLint,
                            width,
                            height,
                            0,
                            gl::RGB,
                            gl::UNSIGNED_BYTE,
                            std::ptr::null(),
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_MIN_FILTER,
                            gl::LINEAR as GLint,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_MAG_FILTER,
                            gl::LINEAR as GLint,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_WRAP_S,
                            gl::CLAMP_TO_BORDER as GLint,
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_WRAP_T,
                            gl::CLAMP_TO_BORDER as GLint,
                        );

                        gl::FramebufferTexture(
                            gl::FRAMEBUFFER,
                            gl::COLOR_ATTACHMENT0 + i as GLuint,
                            buffer,
                            0,
                        );
                        buffer
                    })
                    .collect::<Vec<GLuint>>()
            };

            let mut depth_buffer = 0;
            gl::GenTextures(1, &mut depth_buffer);
            gl::BindTexture(gl::TEXTURE_2D, depth_buffer);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as i32,
                width,
                height,
                0,
                gl::DEPTH_COMPONENT,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_BORDER as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_BORDER as GLint,
            );

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, depth_buffer, 0);

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

            Self { name, buffers }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.name);
        }
    }

    pub fn unbind() {
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

static POST_VERT: &'static str = include_str!("shaders/ambient.light.vert");
static POST_FRAG: &'static str = include_str!("shaders/post.frag");

pub struct Renderer {
    width: i32,
    height: i32,
    g_buffer: Framebuffer,
    light_buffer: Framebuffer,
    post_buffer: Framebuffer,
    model_shader: Shader,
    ambient_shader: Shader,
    directional_shader: Shader,
    post_shader: Shader,
    quad_mesh: Mesh,
    point_shader: Shader,
    point_mesh: Mesh,
}

static RES_FACTOR: i32 = 4;

impl Renderer {
    pub fn new(width: i32, height: i32) -> Self {
        let mut dither_map = 0;
        unsafe {
            gl::GenTextures(1, &mut dither_map);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, dither_map);

            let width = 64;
            let height = 64;
            let depth = 16;

            let size = width * height * depth;
            let data = {
                let mut data: Vec<u8> = Vec::with_capacity(3 * size as usize);
                data.set_len(3 * size as usize);
                let mut rng = rand::thread_rng();
                rng.fill_bytes(&mut data);
                data
            };

            gl::TexImage3D(
                gl::TEXTURE_2D_ARRAY,
                0,
                gl::RGB8 as GLint,
                width,
                height,
                depth,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const std::ffi::c_void,
            );

            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as GLint,
            );
        }
        Self {
            width,
            height,
            g_buffer: {
                let formats = [gl::RGB, gl::RGBA16F, gl::RGB16F];

                Framebuffer::new(width, height, &formats)
            },
            light_buffer: Framebuffer::new(width, height, &[gl::RGB]),
            post_buffer: Framebuffer::new(width / RES_FACTOR, height / RES_FACTOR, &[gl::RGB]),
            model_shader: Shader::from_vert_frag(VS_SRC, FS_SRC),
            ambient_shader: Shader::from_vert_frag(AMBIENT_VERT, AMBIENT_FRAG),
            directional_shader: Shader::from_vert_frag(DIR_VERT, DIR_FRAG),
            post_shader: Shader::from_vert_frag(POST_VERT, POST_FRAG),
            quad_mesh: Mesh::ambient_light(),
            point_shader: Shader::from_vert_frag(POINT_VERT, POINT_FRAG),
            point_mesh: Mesh::point_light(1.0),
        }
    }

    pub fn render(
        &self,
        frame_count: i32,
        camera: &Camera,
        models: &[Model],
        ambient: glm::Vec3,
        directional_lights: &[DirectionalLight],
        point_lights: &[PointLight],
    ) {
        let projection = camera.projection();
        let view = camera.view();
        let view_projection = projection * view;

        self.g_buffer.bind();
        self.model_shader.activate();
        unsafe {
            gl::Enable(gl::DEPTH_TEST);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        };
        for m in models.iter() {
            m.render(&camera);
        }
        self.light_buffer.bind();

        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);

            gl::Clear(gl::COLOR_BUFFER_BIT);
            let buffers = &(self.g_buffer.buffers);
            gl::BindTextures(1, buffers.len() as i32, &buffers[0]);
        }
        self.ambient_shader.activate();
        unsafe {
            gl::Uniform3fv(1, 1, &ambient[0]);
            gl::Uniform1i(2, frame_count + 0);
        }
        self.quad_mesh.draw();

        self.directional_shader.activate();
        unsafe {
            gl::Uniform1i(3, frame_count + 1);
        }
        for light in directional_lights.iter() {
            unsafe {
                gl::Uniform3fv(1, 1, &light.direction[0]);
                gl::Uniform3fv(2, 1, &light.color[0]);
            }
            self.quad_mesh.draw();
        }

        self.point_shader.activate();
        unsafe {
            gl::Uniform1i(5, frame_count + 2);
        };
        for light in point_lights.iter() {
            let &PointLight {
                position,
                color,
                radius,
            } = light;
            let model = num::one();
            let model = glm::ext::translate(&model, position);
            let model = glm::ext::scale(&model, glm::vec3(radius, radius, radius));
            let mvp = view_projection * model;

            unsafe {
                gl::UniformMatrix4fv(1, 1, gl::FALSE, &(mvp[0][0]));

                gl::Uniform3fv(2, 1, &position[0]);
                gl::Uniform3fv(3, 1, &color[0]);
                gl::Uniform1f(4, radius);
            }
            self.point_mesh.draw();
        }

        self.post_buffer.bind();
        self.post_shader.activate();
        unsafe {
            gl::Viewport(0, 0, self.width / RES_FACTOR, self.height / RES_FACTOR);
            gl::Disable(gl::BLEND);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            let buffers = &(self.light_buffer.buffers);
            gl::BindTextures(1, buffers.len() as i32, &buffers[0]);
            gl::Uniform1i(1, RES_FACTOR);
            gl::Uniform1i(2, frame_count + 3);
        }
        self.quad_mesh.draw();
        Shader::deactivate();

        Framebuffer::unbind();
        unsafe {
            gl::Viewport(0, 0, self.width, self.height);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.post_buffer.name);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            gl::BlitFramebuffer(
                0,
                0,
                self.width / RES_FACTOR,
                self.height / RES_FACTOR,
                0,
                0,
                self.width,
                self.height,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }
    }
}
