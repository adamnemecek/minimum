
use super::systems;
use std::sync::Arc;

pub struct MinimumDispatcher {
    world: Arc<systems::World>,
}

impl MinimumDispatcher {
    pub fn new(world: systems::World) -> MinimumDispatcher {
        MinimumDispatcher {
            world: Arc::new(world),
        }
    }

    // Call this to kick off processing.
    pub fn enter_game_loop<F>(self, f: F) -> systems::World
        where
            F: Fn(&MinimumDispatcherContext)
    {
        {
            let ctx = MinimumDispatcherContext {
                world: self.world.clone(),
                should_terminate: std::cell::Cell::new(false)
            };

            loop {
                (f)(&ctx);
                if ctx.should_terminate.get() {
                    break;
                }
            }
        }

        // Then unwrap the world inside it
        let world = Arc::try_unwrap(self.world).unwrap_or_else(|_| {
            unreachable!();
        });

        // Return the world
        world
    }
}

pub struct MinimumDispatcherContext {
    world: Arc<systems::World>,
    should_terminate: std::cell::Cell<bool>
}

//
// Task
//

pub trait Task {
    type RequiredResources: for<'a> super::DataRequirement<'a>
    + Send
    + 'static;

    fn run(&mut self, data: <Self::RequiredResources as super::DataRequirement>::Borrow);
}

impl MinimumDispatcherContext {
    pub fn end_game_loop(&self) {
        self.should_terminate.set(true);
    }

    pub fn world(&self) -> Arc<systems::World> {
        self.world.clone()
    }

    pub fn run_task<T>(&self, mut task: T)
        where T: Task,
    {
        use systems::DataRequirement;
        let required_data = <<T as Task>::RequiredResources>::fetch(&self.world);
        task.run(required_data);
    }
}
