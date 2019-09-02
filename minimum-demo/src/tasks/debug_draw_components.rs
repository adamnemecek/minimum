use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ComponentStorage, ReadComponent, Task, TaskContext};

use crate::components;
use crate::resources::DebugDraw;
use named_type::NamedType;

#[derive(NamedType)]
pub struct DebugDrawComponents;
impl Task for DebugDrawComponents {
    type RequiredResources = (
        Write<DebugDraw>,
        Read<minimum::EntitySet>,
        ReadComponent<components::DebugDrawCircleComponent>,
        ReadComponent<components::DebugDrawRectComponent>,
        ReadComponent<components::PositionComponent>,
    );
    const REQUIRED_FLAGS: usize = framework::context_flags::AUTHORITY_CLIENT as usize;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (mut debug_draw, entity_set, circle_components, rect_components, position_components) =
            data;

        for (entity_index, circle) in circle_components.iter(&entity_set) {
            if let Some(position) = position_components.get(&entity_index) {
                debug_draw.add_circle(position.position(), circle.radius(), circle.color())
            }
        }

        for (entity_index, rect) in rect_components.iter(&entity_set) {
            if let Some(position) = position_components.get(&entity_index) {
                let p0 = position.position() + (rect.size() / 2.0);
                let p1 = position.position() - (rect.size() / 2.0);

                debug_draw.add_rect(p0, p1, rect.color());
            }
        }
    }
}
