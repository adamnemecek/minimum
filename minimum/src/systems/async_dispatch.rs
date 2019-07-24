use std::sync::Arc;

use super::systems;
use crate::async_dispatcher;

use async_dispatcher::{
    AcquiredResourcesLockGuards, Dispatcher, DispatcherBuilder, RequiresResources,
    AcquireCriticalSectionReadLockGuard,
    AcquireCriticalSectionWriteLockGuard,
    acquire_resources as do_acquire_resources,
    acquire_critical_section_read as do_acquire_critical_section_read,
    acquire_critical_section_write as do_acquire_critical_section_write
};

use systems::ResourceId;

//
// Task
//

pub trait Task {
    type RequiredResources: for<'a> systems::DataRequirement<'a>
    + crate::async_dispatcher::RequiresResources<ResourceId>
    + Send
    + 'static;

    fn run(&mut self, data: <Self::RequiredResources as systems::DataRequirement>::Borrow);
}

impl crate::async_dispatcher::ResourceIdTrait for ResourceId {}

//
// Hook up Read/Write to the resource system
//
impl<T: systems::Resource> RequiresResources<ResourceId> for systems::Read<T> {
    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<T: systems::Resource> RequiresResources<ResourceId> for systems::Write<T> {
    fn reads() -> Vec<ResourceId> {
        vec![]
    }
    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
}

impl<T: systems::Resource> RequiresResources<ResourceId> for Option<systems::Read<T>> {
    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<T: systems::Resource> RequiresResources<ResourceId> for Option<systems::Write<T>> {
    fn reads() -> Vec<ResourceId> {
        vec![]
    }
    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
}

//
// Helper that holds the locks and provides a method to fetch the data
//
pub struct AcquiredResources<T>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    _lock_guards: AcquiredResourcesLockGuards<T>,
    world: Arc<systems::TrustCell<systems::World>>,
}

impl<T> AcquiredResources<T>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    pub fn visit<'a, 'b, F>(&'a self, f: F)
    where
        'a : 'b,
        F: FnOnce(T::Borrow),
        T: systems::DataRequirement<'b>,

    {
        let trust_cell_ref = (*self.world).borrow();
        let world_ref = trust_cell_ref.value();
        let fetched = T::fetch(world_ref);
        (f)(fetched);
    }

    //TODO: Could experiment with an api more like fetch from shred
}

//
// Creates a future to acquire the resources needed
//
pub fn acquire_resources<T>(
    dispatcher: Arc<Dispatcher<ResourceId>>,
    world: Arc<systems::TrustCell<systems::World>>,
) -> impl futures::future::Future<Item = AcquiredResources<T>, Error = ()>
where
    T: RequiresResources<ResourceId> + 'static + Send,
{
    use futures::future::Future;

    Box::new(
        do_acquire_resources::<ResourceId, T>(dispatcher).map(move |lock_guards| {
            AcquiredResources {
                _lock_guards: lock_guards,
                world,
            }
        }),
    )
}

pub fn acquire_critical_section_read(
    dispatcher: Arc<Dispatcher<ResourceId>>
) -> impl futures::future::Future<Item = AcquireCriticalSectionReadLockGuard, Error = ()> {
    do_acquire_critical_section_read(dispatcher)
}

pub fn acquire_critical_section_write(
    dispatcher: Arc<Dispatcher<ResourceId>>
) -> impl futures::future::Future<Item = AcquireCriticalSectionWriteLockGuard, Error = ()> {
    do_acquire_critical_section_write(dispatcher)
}

pub struct MinimumDispatcherBuilder {
    dispatcher_builder: DispatcherBuilder<ResourceId>,
    world: systems::World,
}

impl MinimumDispatcherBuilder {
    // Create an empty dispatcher builder
    pub fn new() -> Self {
        MinimumDispatcherBuilder {
            dispatcher_builder: DispatcherBuilder::new(),
            world: systems::World::new(),
        }
    }

    pub fn from_world(world: systems::World) -> MinimumDispatcherBuilder {
        let mut dispatcher_builder = DispatcherBuilder::new();
        for resource in world.keys() {
            dispatcher_builder.register_resource_id(resource.clone());
        }

        MinimumDispatcherBuilder {
            dispatcher_builder,
            world,
        }
    }

    pub fn with_resource<T: systems::Resource>(mut self, resource: T) -> Self {
        self.insert_resource(resource);
        self
    }

    pub fn insert_resource<T: systems::Resource>(&mut self, resource: T) {
        self.world.insert(resource);
        self.dispatcher_builder
            .register_resource_id(ResourceId::new::<T>());
    }

    pub fn world(&self) -> &systems::World {
        &self.world
    }

    // Create the dispatcher
    pub fn build(self) -> MinimumDispatcher {
        let dispatcher = self.dispatcher_builder.build();
        let world = Arc::new(systems::TrustCell::new(self.world));

        MinimumDispatcher {
            dispatcher,
            world
        }
    }
}

pub struct MinimumDispatcher {
    dispatcher: Dispatcher<ResourceId>,
    world: Arc<systems::TrustCell<systems::World>>,
}

impl MinimumDispatcher {
    // Call this to kick off processing.
    pub fn enter_game_loop<F, FutureT>(self, f: F) -> systems::World
    where
        F: Fn(Arc<MinimumDispatcherContext>) -> FutureT + Send + Sync + 'static,
        FutureT: futures::future::Future<Item = (), Error = ()> + Send + 'static,
    {
        let world = self.world.clone();

        self.dispatcher.enter_game_loop(move |dispatcher| {
            let ctx = Arc::new(MinimumDispatcherContext {
                dispatcher: dispatcher.clone(),
                world: world.clone(),
            });

            (f)(ctx)
        });

        // Then unwrap the world inside it
        let world = Arc::try_unwrap(self.world).unwrap_or_else(|_| {
            unreachable!();
        });

        // Return the world
        world.into_inner()
    }
}

pub struct MinimumDispatcherContext {
    dispatcher: Arc<Dispatcher<ResourceId>>,
    world: Arc<systems::TrustCell<systems::World>>,
}

//TODO: I don't like the naming on the member functions here
impl MinimumDispatcherContext {
    pub fn end_game_loop(&self) {
        self.dispatcher.end_game_loop();
    }

    pub fn has_resource<T>(&self) -> bool
    where T: systems::Resource
    {
        (*self.world).borrow().value().has_value::<T>()
    }

    //WARNING: Using the trust cell here is a bit dangerous, it's much
    //safer to use visit_world and visit_world_mut as they appropriately
    //wait to acquire locks to ensure safety
    pub fn world(&self) -> Arc<systems::TrustCell<systems::World>> {
        self.world.clone()
    }

    pub fn run_fn<RequirementT, F>(
        &self,
        f: F,
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        RequirementT: RequiresResources<ResourceId> + 'static + Send,
        F: Fn(AcquiredResources<RequirementT>) + 'static,
    {
        use futures::future::Future;

        Box::new(
            acquire_resources::<RequirementT>(self.dispatcher.clone(), Arc::clone(&self.world)).map(
                move |acquired_resources| {
                    (f)(acquired_resources);
                },
            ),
        )
    }

    pub fn run_task<T>(
        &self,
        mut task: T,
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where
        T: Task,
    {
        use futures::future::Future;

        Box::new(
            acquire_resources::<T::RequiredResources>(self.dispatcher.clone(), Arc::clone(&self.world))
                .map(move |acquired_resources| {
                    acquired_resources.visit(move |resources| {
                        task.run(resources);
                    });
                }),
        )
    }

    //TODO: It would be nice to pass the context into the callback, but need to refactor to use
    //inner arc.
    pub fn visit_world<F>(
        &self,
        f: F
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
        where F : FnOnce(&systems::World)
    {
        use futures::future::Future;

        let world = self.world.clone();

        Box::new(
            acquire_critical_section_read(self.dispatcher.clone())
                .map(move |_acquire_critical_section| {
                    (f)(&(*world).borrow());
                })
        )
    }

    //TODO: It would be nice to pass the context into the callback, but need to refactor to use
    //inner arc.
    pub fn visit_world_mut<F>(
        &self,
        f: F
    ) -> Box<impl futures::future::Future<Item = (), Error = ()>>
    where F : FnOnce(&mut systems::World)
    {
        use futures::future::Future;

        let world = self.world.clone();

        Box::new(
            acquire_critical_section_write(self.dispatcher.clone())
                .map(move |_acquire_critical_section| {
                    (f)(&mut (*world).borrow_mut());
            })
        )
    }
}