
use minimum::systems::{
    simple_dispatch::Task,
    simple_dispatch::MinimumDispatcher,
    Write,
    DataRequirement,
    WorldBuilder
};

//
// This is an example resource. Resources contain data that tasks can operate on.
//
pub struct ExampleResource;
impl ExampleResource {

    pub fn new() -> Self {
        ExampleResource
    }

    pub fn update(&mut self) {
        println!("hello world!");
    }
}

//
// This is an example task. The dispatcher will run it, passing the resources it requires.
//
pub struct ExampleTask;
impl Task for ExampleTask {
    type RequiredResources = (Write<ExampleResource>);

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let mut example_resource = data;
        example_resource.update();
    }
}

//
// In the main loop, you need to:
//
//   1. Register the resources that will be available
//   2. Start the game loop
//   3. Within the game loop, run tasks
//   4. End the loop
//
// The enter_game_loop call will return all the resources.
//
fn main() {
    // Set up a dispatcher with the example resource in it
    let world = WorldBuilder::new()
        .with_resource(ExampleResource::new())
        .build();

    let dispatcher = MinimumDispatcher::new(world);

    // Start the game loop
    dispatcher.enter_game_loop(|ctx| {
        // Run a task, this will call update on the given resource
        ctx.run_task(ExampleTask);

        // Stop the loop
        ctx.end_game_loop();
    });
}
