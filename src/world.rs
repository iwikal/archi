use glm::Mat4;
use specs::{Component, DenseVecStorage, Entity, FlaggedStorage, VecStorage, World, WorldExt};
use specs_hierarchy::Parent;

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
}

use mesh::Mesh;
use specs::{Builder, DispatcherBuilder};
fn add_node(parent: Option<Entity>, node: gltf::Node, world: &mut World, imp: &ImportData) {
    let trans = {
        let arr = node.transform().matrix();
        glm::Mat4::new(
            *glm::Vector4::from_array(&arr[0]),
            *glm::Vector4::from_array(&arr[1]),
            *glm::Vector4::from_array(&arr[2]),
            *glm::Vector4::from_array(&arr[3]),
        )
    };
    let entity = {
        let mut builder = world.create_entity().with(Transform::new(trans));
        if let Some(parent) = parent { builder = builder.with(Child { parent }); }
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                use mesh::ModelVertex;
                let reader = primitive.reader(|buffer| Some(&imp.buffers[buffer.index()]));
                let vertices = {
                    let positions = reader
                        .read_positions()
                        .unwrap_or_else(|| {
                            panic!(
                                "All primitives must have position attributes (mesh: {}, primitive: {})",
                                mesh.index(),
                                primitive.index(),
                            );
                        });
                    positions.map(|pos| {
                        ModelVertex {
                            position: *glm::Vector3::from_array(&pos),
                            normal: glm::vec3(0.0, 0.0, 1.0),
                            color: glm::vec3(0.0, 0.0, 1.0),
                        }
                    }).collect::<Vec<_>>()
                };

                let indices = reader
                    .read_indices()
                    .map(|indices| {
                        indices.into_u32().collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| {
                        panic!("no indices");
                    });

                let primitive = Mesh::new(&vertices, &indices);
                builder = builder.with(primitive);
            }
        }
        builder
    }
    .build();
    for child_node in node.children() {
        add_node(Some(entity), child_node, world, imp);
    }
}

struct RenderSystem { }

use specs::{System, ReadStorage};
impl<'a> System<'a> for RenderSystem {
    type SystemData = (ReadStorage<'a, Transform>,
                       ReadStorage<'a, Mesh>);

    fn run(&mut self, (transforms, meshes): Self::SystemData) {
        use specs::Join;
        for (transform, mesh) in (&transforms, &meshes).join() {
            println!("rendering");
            println!("transform: {:#?}", transform);
            println!("mesh: {:#?}", mesh);
        }
    }
}

pub fn load_world(path: impl AsRef<std::path::Path>) -> World {
    use specs_hierarchy::HierarchySystem;
    let mut world = World::new();
    world.register::<Transform>();
    world.register::<Mesh>();
    let mut dispatcher = DispatcherBuilder::new()
        .with(HierarchySystem::<Child>::new(), "hierarchy_system", &[])
        .with(RenderSystem {}, "render_system", &["hierarchy_system"])
        .build();
    dispatcher.setup(&mut world);

    let (doc, buffers, _) = gltf::import(path).unwrap();

    let imp = ImportData { doc, buffers };
    let scene = imp.doc.scenes().nth(0).unwrap();

    for node in scene.nodes() {
        add_node(None, node, &mut world, &imp);
    }

    dispatcher.dispatch(&mut world);

    world
}
