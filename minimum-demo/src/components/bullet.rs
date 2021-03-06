#[cfg(feature = "editor")]
use imgui_inspect_derive::Inspect;
use crate::base::component::SlabComponentStorage;

// This component contains no data, however an empty component can still be useful to "tag" entities
#[derive(Debug, Clone, Serialize, Deserialize, Default, Inspect)]
pub struct BulletComponent {}

impl BulletComponent {
    pub fn new() -> Self {
        BulletComponent {}
    }
}

impl crate::base::Component for BulletComponent {
    type Storage = SlabComponentStorage<Self>;
}
