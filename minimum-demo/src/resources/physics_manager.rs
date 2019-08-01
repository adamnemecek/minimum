use crate::resources;

const GRAVITY: f32 = 0.0;

//TODO: Are steps really necessary? Appropriate size?
const STEP_SIZE: f32 = 1.0 / 60.0;

pub struct PhysicsManager {
    world: nphysics2d::world::World<f32>,

    time_accumulator: f32,
}

impl PhysicsManager {
    pub fn new() -> Self {
        let mut world = nphysics2d::world::World::<f32>::new();
        let gravity = glm::Vec2::y() * GRAVITY;
        world.set_gravity(gravity);
        world.integration_parameters_mut().dt = STEP_SIZE;

        PhysicsManager {
            world,
            time_accumulator: 0.0,
        }
    }

    pub fn update(&mut self, time_state: &resources::TimeState) {
        let dt = time_state.playing().previous_frame_dt;
        self.time_accumulator += dt;

        let mut steps = 0;
        let accumulated_time = self.time_accumulator;

        let t0 = std::time::Instant::now();
        while self.time_accumulator > STEP_SIZE {
            steps += 1;
            self.world.step();
            self.time_accumulator -= STEP_SIZE;
        }
        let t1 = std::time::Instant::now();

        trace!(
            "update physics took {} in {} steps. Last frame dt: {} time accumulator: {}",
            (t1 - t0).as_micros() as f64 / 1000.0,
            steps,
            dt,
            accumulated_time
        );
    }

    pub fn world(&self) -> &nphysics2d::world::World<f32> {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut nphysics2d::world::World<f32> {
        &mut self.world
    }
}
