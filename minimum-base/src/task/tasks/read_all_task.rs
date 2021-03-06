use std::prelude::v1::*;

use super::ResourceMap;
use super::Task;
use super::TaskConfig;
use super::TaskContextFlags;
use super::TaskFactory;
use super::TrustCell;

use std::marker::PhantomData;

//
// Read All Resources Task
//

/// Simple trait that can be wrapped in a ReadResourceMapTask to get immutable access on the resource
/// map.
pub trait ReadAllTaskImpl: 'static + Send {
    fn configure(config: &mut TaskConfig);
    fn run(context_flags: &TaskContextFlags, resource_map: &ResourceMap);
}

/// Helper struct that configures to read the whole resource map
#[derive(Default)]
pub struct ReadAllTask<T: ReadAllTaskImpl> {
    phantom_data: PhantomData<T>,
}

impl<T: ReadAllTaskImpl> ReadAllTask<T> {
    fn new() -> Self {
        ReadAllTask {
            phantom_data: PhantomData,
        }
    }
}

impl<T: ReadAllTaskImpl> TaskFactory for ReadAllTask<T> {
    fn configure(config: &mut TaskConfig) {
        T::configure(config);

        config.read_all();
    }

    fn create() -> Box<dyn Task> {
        Box::new(Self::new())
    }
}

impl<T: ReadAllTaskImpl + Send> Task for ReadAllTask<T> {
    fn run(&self, context_flags: &TaskContextFlags, resource_map: &TrustCell<ResourceMap>) {
        let resource_map_borrowed = resource_map.borrow();
        T::run(context_flags, &*resource_map_borrowed);
    }
}
