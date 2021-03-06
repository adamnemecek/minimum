use crate::base::resource::{DataRequirement, Read};
use crate::base::{ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::framework::resources::TimeState;

use crate::components;
use crate::base::component::{ReadComponent, WriteComponent};
use crate::base::ComponentStorage;

pub struct UpdatePositionWithVelocity;
pub type UpdatePositionWithVelocityTask = crate::base::ResourceTask<UpdatePositionWithVelocity>;
impl ResourceTaskImpl for UpdatePositionWithVelocity {
    type RequiredResources = (
        Read<crate::base::EntitySet>,
        Read<TimeState>,
        WriteComponent<crate::framework::components::TransformComponent>,
        ReadComponent<crate::framework::components::VelocityComponent>,
        ReadComponent<components::PhysicsBodyComponent>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePrePhysicsGameplay>();
        config.this_provides_data_to::<crate::tasks::PhysicsSyncPreTask>();
        config.run_only_if(crate::framework::context_flags::PLAYMODE_PLAYING);
    }

    fn run(
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            time_state,
            mut transform_components,
            velocity_components,
            physics_body_components,
        ) = data;

        let dt = time_state.playing().previous_frame_dt;

        for (entity, vel) in velocity_components.iter(&entity_set) {
            if physics_body_components.exists(&entity) {
                // Skip any entities that have a physics body as movement is being controlled by
                // nphysics
                continue;
            }

            if let Some(pos) = transform_components.get_mut(&entity) {
                *pos.position_mut() += vel.velocity() * dt;
            }
        }
    }
}
