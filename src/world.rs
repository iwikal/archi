use gl::types::*;
use glm::{Mat4, Vec4};
use material::Material;
use specs::{
    Component, DenseVecStorage, Entity, FlaggedStorage, VecStorage, World,
    WorldExt,
};
use specs_hierarchy::{Hierarchy, Parent};

#[derive(Debug)]
pub struct Transform {
    pub local: Mat4,
    global: Mat4,
    dirty: bool,
}

impl Transform {
    pub fn new(local: Mat4) -> Self {
        Self {
            local,
            global: num::one(),
            dirty: true,
        }
    }
}

impl Component for Transform {
    type Storage = VecStorage<Self>;
}

pub struct Child {
    pub parent: Entity,
}

impl Parent for Child {
    fn parent_entity(&self) -> Entity {
        self.parent
    }
}

impl Component for Child {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

struct ImportData {
    doc: gltf::Document,
    buffers: Vec<gltf::buffer::Data>,
    images: Vec<gl::types::GLuint>,
}

use mesh::Mesh;
use specs::{Builder, Dispatcher, DispatcherBuilder};
fn add_node(
    parent: Option<Entity>,
    node: gltf::Node,
    world: &mut World,
    imp: &ImportData,
) {
    let trans = {
        let arr = node.transform().matrix();
        Mat4::new(
            *Vec4::from_array(&arr[0]),
            *Vec4::from_array(&arr[1]),
            *Vec4::from_array(&arr[2]),
            *Vec4::from_array(&arr[3]),
        )
    };
    let entity = {
        if let Some(mesh) = node.mesh() {
            let mut result = None;
            for primitive in mesh.primitives() {
                let mut builder =
                    world.create_entity().with(Transform::new(trans));
                if let Some(parent) = parent {
                    builder = builder.with(Child { parent });
                }
                let material = primitive.material();
                let (diffuse_texture, uv_set) = match material
                    .pbr_metallic_roughness()
                    .base_color_texture()
                {
                    Some(info) => {
                        let image = info.texture().source();
                        let diff_name = *imp.images.get(image.index()).unwrap();
                        let uv_set = info.tex_coord();
                        (diff_name, uv_set)
                    }
                    None => (0, 0),
                };
                let normal_texture = match material.normal_texture() {
                    Some(normal) => {
                        let image = normal.texture().source();
                        *imp.images.get(image.index()).unwrap()
                    }
                    None => 0,
                };
                let material = Material {
                    diffuse_texture,
                    normal_texture,
                };
                builder = builder.with(material);
                use mesh::ModelVertex;
                let reader = primitive
                    .reader(|buffer| Some(&imp.buffers[buffer.index()]));
                let vertices = {
                    let positions =
                        reader.read_positions().unwrap_or_else(|| {
                            panic!(
                                "No positions (mesh: {}, primitive: {})",
                                mesh.index(),
                                primitive.index(),
                            );
                        });
                    let normals = reader.read_normals().unwrap_or_else(|| {
                        panic!(
                            "No normals (mesh: {}, primitive: {})",
                            mesh.index(),
                            primitive.index(),
                        );
                    });
                    let tangents = reader.read_tangents().unwrap_or_else(|| {
                        panic!(
                            "No tangents (mesh: {}, primitive: {})",
                            mesh.index(),
                            primitive.index(),
                        );
                    });
                    let mut source_uvs = reader
                        .read_tex_coords(uv_set)
                        .map(|uvs| uvs.into_f32());
                    let uvs = std::iter::from_fn(|| match &mut source_uvs {
                        Some(iter) => iter.next(),
                        None => Some([0.0, 0.0]),
                    });
                    positions
                        .zip(normals)
                        .zip(tangents)
                        .zip(uvs)
                        .map(|(((pos, norm), tan), uv)| ModelVertex {
                            position: *glm::Vec3::from_array(&pos),
                            normal: *glm::Vec3::from_array(&norm),
                            tangent: glm::vec3(tan[0], tan[1], tan[2]),
                            uv: *glm::Vec2::from_array(&uv),
                        })
                        .collect::<Vec<_>>()
                };

                let indices = reader
                    .read_indices()
                    .unwrap_or_else(|| {
                        panic!(
                            "No positions (mesh: {}, primitive: {})",
                            mesh.index(),
                            primitive.index(),
                        );
                    })
                    .into_u32()
                    .collect::<Vec<_>>();

                let primitive = Mesh::new(&vertices, &indices);
                result = Some(builder.with(primitive).build());
            }
            result.unwrap_or_else(|| {
                panic!("No primitives (mesh: {})", mesh.index(),);
            })
        } else {
            let mut builder = world.create_entity().with(Transform::new(trans));
            if let Some(parent) = parent {
                builder = builder.with(Child { parent });
            }
            builder.build()
        }
    };
    for child_node in node.children() {
        add_node(Some(entity), child_node, world, imp);
    }
}

use std::num::Wrapping;
struct RenderSystem {
    frame_count: Wrapping<i32>,
}

impl RenderSystem {
    fn new() -> Self {
        Self {
            frame_count: Wrapping(0),
        }
    }
}

use crate::camera::Camera;
use crate::renderer::Renderer;
use specs::{shred::PanicHandler, Read, ReadStorage, System, WriteStorage};
impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        Read<'a, Renderer, PanicHandler>,
        Read<'a, Camera, PanicHandler>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, Material>,
    );

    fn run(
        &mut self,
        (renderer, camera, transforms, meshes, materials): Self::SystemData,
    ) {
        use specs::Join;
        let models = (&transforms, &meshes, &materials).join().map(
            |(transform, mesh, material)| (&transform.global, mesh, material),
        );

        let ambient_color = {
            let brightness = 1.0 / 1024.0;
            glm::vec3(brightness, brightness, brightness)
        };

        let point_lights = {
            use glm::vec3;
            use renderer::PointLight as Light;
            [
                Light {
                    radius: 3.0,
                    position: vec3(-1.3, 2.5, 0.),
                    color: vec3(0.5, 0.5, 0.4) / 16.0,
                },
                Light {
                    radius: 3.0,
                    position: vec3(0., 2.5, 0.),
                    color: vec3(0.5, 0.5, 0.4) / 16.0,
                },
                Light {
                    radius: 3.0,
                    position: vec3(1.3, 2.5, 0.),
                    color: vec3(0.5, 0.5, 0.4) / 16.0,
                },
                Light {
                    radius: 3.0,
                    position: vec3(0., -0.5, 2.),
                    color: vec3(0.5, 0.5, 1.) / 16.0,
                },
                Light {
                    radius: 3.0,
                    position: vec3(0., -0.5, -2.),
                    color: vec3(0.5, 0.5, 1.) / 16.0,
                },
                Light {
                    radius: 3.0,
                    position: vec3(3.5, 4.3, 1.),
                    color: vec3(1., 0., 0.) / 16.0,
                },
            ];
            []
        };

        let dir_lights = {
            use glm::vec3;
            use renderer::DirectionalLight as Light;
            [
                Light {
                    direction: vec3(-1., -1., 1.),
                    color: vec3(0.66, 0.66, 1.0) / 16.0,
                },
                Light {
                    direction: vec3(10., 0.5, -1.),
                    color: vec3(1.0, 1.0, 0.5) / 4.0,
                },
            ]
        };

        renderer.render(
            self.frame_count.0,
            &camera,
            models,
            ambient_color,
            &dir_lights,
            &point_lights,
        );

        if renderer.temporal_dither {
            self.frame_count += Wrapping(1);
        }
    }
}

use gltf::image::{Data, Format};
fn load_texture(data: Data) -> gl::types::GLuint {
    let Data {
        pixels,
        format,
        width,
        height,
    } = data;
    let mut name = 0;
    unsafe {
        gl::GenTextures(1, &mut name);
        gl::BindTexture(gl::TEXTURE_2D, name);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            match format {
                Format::R8 => gl::R8,
                Format::R8G8 => gl::RG8,
                Format::R8G8B8 => gl::RGB8,
                Format::R8G8B8A8 => gl::RGBA8,
                Format::B8G8R8 => gl::RGB8,
                Format::B8G8R8A8 => gl::RGBA8,
            } as GLint,
            width as GLint,
            height as GLint,
            0,
            match format {
                Format::R8 => gl::RED,
                Format::R8G8 => gl::RG,
                Format::R8G8B8 | Format::B8G8R8 => gl::RGB,
                Format::R8G8B8A8 | Format::B8G8R8A8 => gl::RGBA,
            },
            gl::UNSIGNED_BYTE,
            pixels.as_ptr() as *const std::ffi::c_void,
        );
        // FIXME parameters
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
    }
    name
}

struct TransformSystem;

impl TransformSystem {
    fn new() -> Self {
        TransformSystem
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Read<
            'a,
            Hierarchy<Child>,
            specs_hierarchy::HierarchySetupHandler<Child>,
        >,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (hierarchy, mut transforms): Self::SystemData) {
        let mut f = |entity: Entity, global: Mat4| {
            let mut transform =
                transforms.get_mut(entity).expect("Must have transform");
            if transform.dirty {
                transform.global = global * transform.local;
                transform.dirty = false;
                for &child in hierarchy.children(entity) {
                    transforms
                        .get_mut(child)
                        .expect("Must have transform")
                        .dirty = true;
                }
            }
            transforms.get(entity).expect("Must have transform").global
        };

        for &child in hierarchy.all() {
            let parent = hierarchy.parent(child).expect("No orphans");
            let global = f(parent, num::one());
            f(child, global);
        }
    }
}

pub fn load_world(
    path: impl AsRef<std::path::Path>,
    renderer: Renderer,
    camera: Camera,
) -> (World, Dispatcher<'static, 'static>) {
    use specs_hierarchy::HierarchySystem;
    let mut world = World::new();
    world.register::<Transform>();
    world.register::<Mesh>();
    world.register::<Material>();
    world.insert(renderer);
    world.insert(camera);
    let mut dispatcher = DispatcherBuilder::new()
        .with(HierarchySystem::<Child>::new(), "hierarchy_system", &[])
        .with(
            TransformSystem::new(),
            "transform_system",
            &["hierarchy_system"],
        )
        .with(
            RenderSystem::new(),
            "render_system",
            &["transform_system", "hierarchy_system"],
        )
        .build();
    dispatcher.setup(&mut world);

    let (doc, buffers, images) = gltf::import(path).unwrap();

    let images = images
        .into_iter()
        .map(|image| load_texture(image))
        .collect::<Vec<_>>();

    let imp = ImportData {
        doc,
        buffers,
        images,
    };
    let scene = imp.doc.scenes().nth(0).unwrap();

    for node in scene.nodes() {
        add_node(None, node, &mut world, &imp);
    }

    (world, dispatcher)
}
