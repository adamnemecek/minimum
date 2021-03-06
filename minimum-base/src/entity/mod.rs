//! This module allows creating entities and associating components with the entities.
//!
//! Components are automatically cleared when entities are destroyed.

use crate::slab::GenSlabKey;

mod entity;
mod entity_factory;
mod entity_set;

pub use entity::Entity;
pub use entity::EntityRef;
pub use entity_factory::BasicEntityPrototype;
pub use entity_factory::EntityFactory;
pub use entity_factory::EntityPrototype;
pub use entity_set::EntitySet;

mod pending_delete;
pub use pending_delete::PendingDeleteComponent;

pub type EntityHandle = GenSlabKey<Entity>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component;
    use crate::component::ComponentRegistry;
    use crate::resource;
    use component::Component;
    use component::ComponentStorage;
    use component::VecComponentStorage;

    struct TestComponent {
        _value: i32,
    }

    impl TestComponent {
        fn new(value: i32) -> Self {
            TestComponent { _value: value }
        }
    }

    impl Component for TestComponent {
        type Storage = VecComponentStorage<Self>;
    }

    #[test]
    fn test_entity_count() {
        let mut resource_map = resource::ResourceMap::new();
        let mut component_registry = ComponentRegistry::new();
        component_registry.register_component::<TestComponent>();
        let mut entity_set = EntitySet::new(component_registry);
        resource_map.insert(<TestComponent as Component>::Storage::new());
        resource_map.insert(<PendingDeleteComponent as Component>::Storage::new());

        let entity = entity_set.allocate();
        assert_eq!(entity_set.entity_count(), 1);
        entity_set.enqueue_free(
            &entity,
            &mut *resource_map.fetch_mut::<<PendingDeleteComponent as Component>::Storage>(),
        );
        assert_eq!(entity_set.entity_count(), 1);
        entity_set.flush_free(&resource_map);
        assert_eq!(entity_set.entity_count(), 0);
    }

    #[test]
    fn test_destroy_entity_releases_components() {
        // Save on typing..
        type Storage = <self::TestComponent as Component>::Storage;

        let mut component_registry = ComponentRegistry::new();
        component_registry.register_component::<TestComponent>();

        let mut resource_map = resource::ResourceMap::new();
        let mut entity_set = EntitySet::new(component_registry);
        resource_map.insert(<TestComponent as Component>::Storage::new());
        resource_map.insert(<PendingDeleteComponent as Component>::Storage::new());

        // Create an entity
        let entity_handle = entity_set.allocate();
        let entity = entity_set.get_entity_ref(&entity_handle).unwrap();

        // Add the component
        {
            let mut test_component_storage = resource_map.fetch_mut::<Storage>();
            entity.add_component(&mut *test_component_storage, TestComponent::new(1)).unwrap();
        }

        // Ensure after we enqueue free and flush free, the component is released
        entity_set.enqueue_free(
            &entity_handle,
            &mut *resource_map.fetch_mut::<<PendingDeleteComponent as Component>::Storage>(),
        );
        assert!(resource_map
            .fetch::<Storage>()
            .get(&entity_handle)
            .is_some());
        entity_set.flush_free(&resource_map);
        assert!(resource_map
            .fetch::<Storage>()
            .get(&entity_handle)
            .is_none());
    }

    #[test]
    fn test_add_get_component() {
        // Save on typing..
        type Storage = <self::TestComponent as Component>::Storage;

        let mut component_registry = ComponentRegistry::new();
        component_registry.register_component::<TestComponent>();

        let mut resource_map = resource::ResourceMap::new();
        let mut entity_set = EntitySet::new(component_registry);
        resource_map.insert(<TestComponent as Component>::Storage::new());

        // Create an entity
        let entity_handle = entity_set.allocate();
        let entity = entity_set.get_entity_ref(&entity_handle).unwrap();

        let mut test_component_storage = resource_map.fetch_mut::<Storage>();

        // Fail to find the component
        let component = entity.get_component::<TestComponent>(&test_component_storage);
        assert!(component.is_none());

        // Add the component
        entity.add_component(&mut *test_component_storage, TestComponent::new(1)).unwrap();

        // Succeed in finding the component
        let component = entity.get_component::<TestComponent>(&test_component_storage);
        assert!(component.is_some());

        //TODO: Had to drop remove_component for now since it doesn't call the free handler
        /*
        // Remove the component
        entity.remove_component::<TestComponent>(&mut test_component_storage);

        // Fail to find the component
        let component = entity.get_component::<TestComponent>(&test_component_storage);
        assert!(component.is_none());
        */
    }
}
