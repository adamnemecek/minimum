#[cfg(feature = "editor")]
use crate::inspect::common_types::*;

#[cfg(feature = "editor")]
use crate::select::SelectableComponentPrototype;

use imgui_inspect_derive::Inspect;
use base::component::SlabComponentStorage;
use crate::FrameworkEntityPrototypeInner;
use crate::components::TransformComponentPrototype;

#[derive(Debug, Clone, Serialize, Deserialize, Inspect)]
pub struct DebugDrawCircleComponent {
    #[inspect_slider(min_value = 0.1, max_value = 100.0)]
    radius: f32,

    #[inspect(proxy_type = "ImGlmColor4")]
    color: glm::Vec4,
}

impl Default for DebugDrawCircleComponent {
    fn default() -> Self {
        DebugDrawCircleComponent {
            radius: 10.0,
            color: glm::vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl DebugDrawCircleComponent {
    pub fn new(radius: f32, color: glm::Vec4) -> Self {
        DebugDrawCircleComponent { radius, color }
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn color(&self) -> glm::Vec4 {
        self.color
    }
}

#[cfg(feature = "editor")]
impl SelectableComponentPrototype<Self> for DebugDrawCircleComponent {
    fn create_selection_shape(
        framework_entity: &FrameworkEntityPrototypeInner,
        data: &Self,
    ) -> (
        ncollide::math::Isometry<f32>,
        ncollide::shape::ShapeHandle<f32>,
    ) {
        let mut scale = 1.0;
        if let Some(transform) = framework_entity.find_component_prototype::<TransformComponentPrototype>() {
            scale = transform.data().uniform_scale();
        }

        let mut radius = data.radius * scale;
        if radius < std::f32::MIN_POSITIVE {
            warn!("Tried to create a circle with <=0 radius");
            radius = std::f32::MIN_POSITIVE;
        }

        use ncollide::shape::{Ball, ShapeHandle};
        (
            ncollide::math::Isometry::<f32>::new(glm::zero(), glm::zero()),
            ShapeHandle::new(Ball::new(radius)),
        )
    }
}

impl base::Component for DebugDrawCircleComponent {
    type Storage = SlabComponentStorage<Self>;
}
