use base::component::VecComponentStorage;
use base::component::{ComponentCreateQueueFlushListener, ComponentStorage};
use base::Component;
use base::ComponentFactory;
use base::ComponentPrototype;
use base::EntityHandle;
use base::EntitySet;
use base::ResourceMap;

use ncollide::world::CollisionObjectHandle;

use crate::FrameworkComponentPrototype;
use ncollide::shape::ShapeHandle;
use ncollide::world::{CollisionGroups, GeometricQueryType};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct EditorShapeComponent {
    shape_handle: ShapeHandle<f32>,
    collider_handle: CollisionObjectHandle,
}

impl EditorShapeComponent {
    pub fn new(shape_handle: ShapeHandle<f32>, collider_handle: CollisionObjectHandle) -> Self {
        EditorShapeComponent {
            shape_handle,
            collider_handle,
        }
    }

    pub fn collider_handle(&self) -> &CollisionObjectHandle {
        &self.collider_handle
    }

    pub fn shape_handle(&self) -> &ShapeHandle<f32> {
        &self.shape_handle
    }
}

impl base::Component for EditorShapeComponent {
    type Storage = VecComponentStorage<Self>;
}

//
// The free handler ensures that when an entity is destroyed, its body components get cleaned up
//
pub struct EditorShapeComponentFreeHandler {}

impl base::component::ComponentFreeHandler<EditorShapeComponent>
    for EditorShapeComponentFreeHandler
{
    fn on_entities_free(
        resource_map: &base::ResourceMap,
        entity_handles: &[base::EntityHandle],
        storage: &mut <EditorShapeComponent as Component>::Storage,
    ) {
        let mut editor_collision_world =
            resource_map.fetch_mut::<crate::resources::editor::EditorCollisionWorld>();
        let physics_world: &mut ncollide::world::CollisionWorld<f32, EntityHandle> =
            editor_collision_world.world_mut();

        for entity_handle in entity_handles {
            if let Some(c) = storage.get_mut(&entity_handle) {
                physics_world.remove(&[c.collider_handle]);
            }
        }
    }
}

//
// Creates a component
//
#[derive(Clone)]
pub struct EditorShapeComponentPrototype {
    shape_handle: ncollide::shape::ShapeHandle<f32>,
}

impl EditorShapeComponentPrototype {
    pub fn new(shape_handle: ncollide::shape::ShapeHandle<f32>) -> Self {
        EditorShapeComponentPrototype { shape_handle }
    }
}

impl ComponentPrototype for EditorShapeComponentPrototype {
    type Factory = EditorShapeComponentFactory;
}

impl FrameworkComponentPrototype for EditorShapeComponentPrototype {
    fn component_type() -> std::any::TypeId {
        std::any::TypeId::of::<EditorShapeComponent>()
    }
}

//
// Factory for PhysicsBody components
//
pub struct EditorShapeComponentFactory {
    prototypes: VecDeque<(EntityHandle, EditorShapeComponentPrototype)>,
}

impl EditorShapeComponentFactory {
    pub fn new() -> Self {
        EditorShapeComponentFactory {
            prototypes: VecDeque::new(),
        }
    }
}

impl ComponentFactory<EditorShapeComponentPrototype> for EditorShapeComponentFactory {
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &EditorShapeComponentPrototype,
    ) {
        self.prototypes
            .push_back((entity_handle.clone(), prototype.clone()));
    }
}

impl ComponentCreateQueueFlushListener for EditorShapeComponentFactory {
    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        let mut collision_world =
            resource_map.fetch_mut::<crate::resources::editor::EditorCollisionWorld>();
        let mut storage = resource_map.fetch_mut::<<EditorShapeComponent as Component>::Storage>();
        for (entity_handle, data) in self.prototypes.drain(..) {
            if let Some(entity) = entity_set.get_entity_ref(&entity_handle) {

                let collider = collision_world.world_mut().add(
                    ncollide::math::Isometry::new(glm::zero(), glm::zero()),
                    data.shape_handle.clone(),
                    CollisionGroups::new(),
                    GeometricQueryType::Proximity(0.001),
                    entity_handle,
                );

                entity.add_component(
                    &mut *storage,
                    EditorShapeComponent::new(data.shape_handle, collider.handle()),
                ).unwrap();
            }
        }
    }
}
