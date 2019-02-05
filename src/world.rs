use glm::Mat4;
use specs::{Component, DenseVecStorage, Entity, FlaggedStorage, VecStorage, World};
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

pub fn load_world() -> World {
    use specs::{Builder, DispatcherBuilder};
    use specs_hierarchy::HierarchySystem;
    let mut world = World::new();
    world.register::<Transform>();
    let mut dispatcher = DispatcherBuilder::new()
        .with(HierarchySystem::<Child>::new(), "hierarchy_system", &[])
        .build();
    dispatcher.setup(&mut world.res);

    let e0 = world
        .create_entity()
        .with(Transform::new(num::one()))
        .build();
    world
        .create_entity()
        .with(Transform::new(num::one()))
        .with(Child { parent: e0 })
        .build();

    dispatcher.dispatch(&mut world.res);

    world
}
