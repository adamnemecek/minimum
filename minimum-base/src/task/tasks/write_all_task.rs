use std::prelude::v1::*;

use super::ResourceMap;
use super::Task;
use super::TaskConfig;
use super::TaskContextFlags;
use super::TaskFactory;
use super::TrustCell;

use std::marker::PhantomData;

/// Simple trait that can be wrapped in a WriteAllTask to get mutable access on the resource
/// map.
pub trait WriteAllTaskImpl: 'static + Send {
    fn configure(config: &mut TaskConfig);
    fn run(context_flags: &TaskContextFlags, resource_map: &mut ResourceMap);
}

/// Helper struct that configures/fetches resources automatically
#[derive(Default)]
pub struct WriteAllTask<T: WriteAllTaskImpl> {
    phantom_data: PhantomData<T>,
}

impl<T: WriteAllTaskImpl> WriteAllTask<T> {
    fn new() -> Self {
        WriteAllTask {
            phantom_data: PhantomData,
        }
    }
}

impl<T: WriteAllTaskImpl> TaskFactory for WriteAllTask<T> {
    fn configure(config: &mut TaskConfig) {
        T::configure(config);

        config.write_all();
    }

    fn create() -> Box<dyn Task> {
        Box::new(Self::new())
    }
}

impl<T: WriteAllTaskImpl + Send> Task for WriteAllTask<T> {
    fn run(&self, context_flags: &TaskContextFlags, resource_map: &TrustCell<ResourceMap>) {
        let mut resource_map_borrowed = resource_map.borrow_mut();
        T::run(context_flags, &mut *resource_map_borrowed);
    }
}
