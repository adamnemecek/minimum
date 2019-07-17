use super::Component;
use super::ComponentStorage;
use super::EntityHandle;

pub struct VecComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<T>)>,
{
    slab_iter: I,
    entity_set: &'a super::entity::EntitySet,
}

impl<'a, T, I> VecComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<T>)>,
{
    fn new(entity_set: &'a super::entity::EntitySet, slab_iter: I) -> Self {
        VecComponentIterator {
            entity_set,
            slab_iter,
        }
    }
}

impl<'a, T, I> Iterator for VecComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<T>)>,
{
    type Item = (EntityHandle, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.slab_iter.next().map(|(entitiy_index, component)| {
            (
                self.entity_set
                    .upgrade_index_to_handle(entitiy_index as u32),
                component.as_ref().unwrap(),
            )
        })
    }
}

pub struct VecComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a mut Option<T>)>,
{
    slab_iter: I,
    entity_set: &'a super::entity::EntitySet,
}

impl<'a, T, I> VecComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a mut Option<T>)>,
{
    fn new(entity_set: &'a super::entity::EntitySet, slab_iter: I) -> Self {
        VecComponentIteratorMut {
            entity_set,
            slab_iter,
        }
    }
}

impl<'a, T, I> Iterator for VecComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a mut Option<T>)>,
{
    type Item = (EntityHandle, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.slab_iter.next().map(|(entitiy_index, component)| {
            (
                self.entity_set
                    .upgrade_index_to_handle(entitiy_index as u32),
                component.as_mut().unwrap(),
            )
        })
    }
}

pub struct VecComponentStorage<T: Component> {
    components: Vec<Option<T>>,
}

impl<T: Component> VecComponentStorage<T> {
    pub fn new() -> Self {
        VecComponentStorage::<T> {
            components: Vec::with_capacity(32), //TODO: Hardcoded value
        }
    }

    pub fn iter<'a>(
        &'a self,
        entity_set: &'a super::entity::EntitySet,
    ) -> impl Iterator<Item = (EntityHandle, &'a T)> {
        VecComponentIterator::<T, _>::new(
            entity_set,
            self.components
                .iter()
                .enumerate()
                .filter(|(_entity_index, component_key)| component_key.is_some()),
        )
    }

    pub fn iter_mut<'a>(
        &'a mut self,
        entity_set: &'a super::entity::EntitySet,
    ) -> impl Iterator<Item = (EntityHandle, &'a mut T)> {
        VecComponentIteratorMut::<T, _>::new(
            entity_set,
            self.components
                .iter_mut()
                .enumerate()
                .filter(|(_entity_index, component_key)| component_key.is_some()),
        )
    }
}

impl<T: Component> ComponentStorage<T> for VecComponentStorage<T> {
    fn allocate(&mut self, entity: &EntityHandle, data: T) {
        // If the slab keys vec isn't long enough, expand it
        if self.components.len() <= entity.index() as usize {
            // Can't use resize() because T is not guaranteed to be cloneable
            self.components.reserve(entity.index() as usize + 1);
            for _index in self.components.len()..(entity.index() as usize + 1) {
                self.components.push(None);
            }
        }

        assert!(self.components[entity.index() as usize].is_none());
        self.components[entity.index() as usize] = Some(data);
    }

    fn free(&mut self, entity: &EntityHandle) {
        assert!(self.components[entity.index() as usize].is_some());
        self.components[entity.index() as usize] = None;
    }

    fn free_if_exists(&mut self, entity: &EntityHandle) {
        if entity.index() as usize >= self.components.len() {
            return;
        }

        if self.components[entity.index() as usize].is_some() {
            self.free(entity);
        }
    }

    fn get(&self, entity: &EntityHandle) -> Option<&T> {
        if entity.index() as usize >= self.components.len() {
            return None;
        }

        self.components[entity.index() as usize].as_ref()
    }

    fn get_mut(&mut self, entity: &EntityHandle) -> Option<&mut T> {
        if entity.index() as usize >= self.components.len() {
            return None;
        }

        self.components[entity.index() as usize].as_mut()
    }
}
