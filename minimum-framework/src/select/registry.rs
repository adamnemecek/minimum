use super::SelectableComponentPrototype;
use crate::FrameworkComponentPrototypeDyn;
use hashbrown::HashMap;
use std::marker::PhantomData;
use crate::prototype::FrameworkEntityPrototypeInner;

trait RegisteredComponentPrototypeTrait: Send + Sync {
    fn create_selection_shape(
        &self,
        framework_entity: &FrameworkEntityPrototypeInner,
        component_prototype: &dyn FrameworkComponentPrototypeDyn,
    ) -> (
        ncollide::math::Isometry<f32>,
        ncollide::shape::ShapeHandle<f32>,
    );
}

struct RegisteredComponentPrototype<T> {
    phantom_data: PhantomData<T>,
}

impl<T> RegisteredComponentPrototype<T> {
    fn new() -> Self {
        RegisteredComponentPrototype {
            phantom_data: PhantomData,
        }
    }
}

impl<T> RegisteredComponentPrototypeTrait for RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototypeDyn + SelectableComponentPrototype<T>,
{
    fn create_selection_shape(
        &self,
        framework_entity: &FrameworkEntityPrototypeInner,
        component_prototype: &dyn FrameworkComponentPrototypeDyn,
    ) -> (
        ncollide::math::Isometry<f32>,
        ncollide::shape::ShapeHandle<f32>,
    ) {
        let t = component_prototype.downcast_ref::<T>().unwrap();
        <T as SelectableComponentPrototype<T>>::create_selection_shape(framework_entity, t)
    }
}

//
// ComponentRegistry
//
pub struct SelectRegistry {
    registered_component_prototypes:
        HashMap<std::any::TypeId, Box<dyn RegisteredComponentPrototypeTrait>>,
}

impl SelectRegistry {
    pub fn new() -> Self {
        SelectRegistry {
            registered_component_prototypes: HashMap::new(),
        }
    }

    pub fn register_component_prototype<
        T: FrameworkComponentPrototypeDyn + SelectableComponentPrototype<T>,
    >(
        &mut self,
    ) {
        self.registered_component_prototypes.insert(
            std::any::TypeId::of::<T>(),
            Box::new(RegisteredComponentPrototype::<T>::new()),
        );
    }

    pub fn create_selection_shape(
        &self,
        framework_entity: &FrameworkEntityPrototypeInner,
        component_prototype: &dyn FrameworkComponentPrototypeDyn,

    ) -> Option<(
        ncollide::math::Isometry<f32>,
        ncollide::shape::ShapeHandle<f32>,
    )> {
        let component_prototype_type = FrameworkComponentPrototypeDyn::type_id(component_prototype);

        if let Some(registered) = self
            .registered_component_prototypes
            .get(&component_prototype_type)
        {
            Some(registered.create_selection_shape(framework_entity, component_prototype))
        } else {
            None
        }
    }
}
