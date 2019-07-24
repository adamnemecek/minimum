use minimum::component::SlabComponentStorage;

use crate::resources::TimeState;

// This component contains no data, however an empty component can still be useful to "tag" entities
#[derive(Debug)]
pub struct BulletComponent {
}

impl BulletComponent {
    pub fn new() -> Self {
        BulletComponent {
        }
    }
}

impl minimum::Component for BulletComponent {
    type Storage = SlabComponentStorage<BulletComponent>;
}