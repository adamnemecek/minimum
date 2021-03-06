use crate::base::component::{ComponentCreateQueueFlushListener, SlabComponentStorage};
use crate::base::ComponentFactory;
use crate::base::ComponentPrototype;
use crate::base::EntityHandle;
use crate::base::EntitySet;
use crate::base::ResourceMap;
use crate::base::{Component, ComponentStorage};
use crate::framework::components::transform;

use nphysics::object::BodyHandle;
use nphysics::object::ColliderDesc;
use nphysics::object::RigidBodyDesc;

use crate::framework::components::TransformComponent;
use crate::framework::FrameworkComponentPrototypeDyn;
use crate::framework::FrameworkComponentPrototype;
use std::collections::VecDeque;
use crate::framework::FrameworkEntityPrototypeInner;

#[cfg(feature = "editor")]
use crate::framework::select::SelectableComponentPrototype;

use crate::framework::components::TransformComponentPrototype;
#[cfg(feature = "dim3")]
use nalgebra::UnitQuaternion;

#[derive(Debug, Inspect)]
pub struct PhysicsBodyComponent {
    #[inspect(skip)]
    body_handle: BodyHandle,
}

//
// Holds a handle to a physics body
// - It's always possible to directly create this component using a body_handle, but you need to have
//   a write lock on the component storage
// - If you'd rather defer construction, you could instead create a PhysicsBodyComponentPrototype, which
//   takes an nphysics RigidBodyDesc. The PhysicsBodyComponentFactory will create the body for you
//   and attach it to the entity
// - If the entity owning the component is cleaned up, PhysicsBodyComponentFreeHandler will handle
//   destroying the physics body
//
impl PhysicsBodyComponent {
    pub fn new(body_handle: BodyHandle) -> Self {
        PhysicsBodyComponent { body_handle }
    }

    pub fn body_handle(&self) -> BodyHandle {
        self.body_handle
    }
}

impl crate::base::Component for PhysicsBodyComponent {
    type Storage = SlabComponentStorage<Self>;
}

//
// The free handler ensures that when an entity is destroyed, its body components get cleaned up
//
pub struct PhysicsBodyComponentFreeHandler {}

impl crate::base::component::ComponentFreeHandler<PhysicsBodyComponent>
    for PhysicsBodyComponentFreeHandler
{
    fn on_entities_free(
        resource_map: &crate::base::ResourceMap,
        entity_handles: &[crate::base::EntityHandle],
        storage: &mut <PhysicsBodyComponent as Component>::Storage,
    ) {
        let mut physics_manager = resource_map.fetch_mut::<crate::resources::PhysicsManager>();
        let physics_world: &mut nphysics::world::World<f32> = physics_manager.world_mut();

        for entity_handle in entity_handles {
            if let Some(c) = storage.get_mut(&entity_handle) {
                physics_world.remove_bodies(&[c.body_handle]);
            }
        }
    }
}

//
// This is a wrapper for RigidBodyDesc. RigidBodyDesc requires a lifetime and holds refs to
// ColliderDescs. I think it will be a bit easier to allow the body to psuedo-own its own colliders.
// This object owns the RigidBodyDesc and each ColliderDesc, and allows the RigidBodyDesc to have
// references to the colliders.
//
// Another approach would probably be to make separate, more serialization-friendly description
// structures. This has the added advantage that these objects could be expressed in data files.
//
pub struct PhysicsBodyComponentDesc {
    // Some unsafe code in here assumes ColliderDesc is pinned.
    //TODO: Can we actually use Pin?
    #[allow(clippy::box_vec)]
    collider_desc: Vec<Box<ColliderDesc<f32>>>,
    rigid_body_desc: RigidBodyDesc<'static, f32>,
}

impl PhysicsBodyComponentDesc {
    pub fn new(rigid_body_desc: RigidBodyDesc<'static, f32>) -> Self {
        PhysicsBodyComponentDesc {
            collider_desc: Vec::new(),
            rigid_body_desc,
        }
    }

    pub fn add_collider(&mut self, collider_desc: ColliderDesc<f32>) {
        // Box the collider
        let collider_boxed = Box::new(collider_desc);

        // Turn the box into a box and a ref to the same box. The new box will solely be responsible
        // for cleaning up the memory. The reference will be given to the body. Because the
        // collider and body desc have the same lifetimes, the reference will always be valid for
        // the lifetime of the body description
        let collider_ptr: *mut ColliderDesc<f32> = Box::into_raw(collider_boxed);
        let collider_ref = unsafe { &*collider_ptr };
        let collider_boxed = unsafe { Box::from_raw(collider_ptr) };

        // Push the box to ensure the memory gets cleaned up when this struct is cleaned up
        self.collider_desc.push(collider_boxed);
        self.rigid_body_desc.add_collider(collider_ref);
    }

    pub fn clone_rigid_body_desc(&self) -> RigidBodyDesc<f32> {
        self.rigid_body_desc.clone()
    }
}

//
// Custom Prototype
//
#[derive(Clone, Inspect)]
pub struct PhysicsBodyComponentPrototypeCustom {
    #[inspect(skip)]
    desc: std::sync::Arc<PhysicsBodyComponentDesc>,
}

impl PhysicsBodyComponentPrototypeCustom {
    pub fn new(desc: PhysicsBodyComponentDesc) -> Self {
        PhysicsBodyComponentPrototypeCustom {
            desc: std::sync::Arc::new(desc),
        }
    }
}

impl ComponentPrototype for PhysicsBodyComponentPrototypeCustom {
    type Factory = PhysicsBodyComponentFactory;
}

impl FrameworkComponentPrototypeDyn for PhysicsBodyComponentPrototypeCustom {
    fn component_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<PhysicsBodyComponent>()
    }
}

type RectSize = crate::framework::components::transform::Scale;
#[cfg(feature = "editor")]
type ImRectSize = crate::framework::components::transform::ImScale;

//
// Box Prototype
//
#[derive(Clone, Serialize, Deserialize, Inspect)]
pub struct PhysicsBodyComponentPrototypeBox {
    #[inspect(proxy_type = "ImRectSize")]
    size: RectSize,
    mass: f32,
    collision_group_membership: u32,
    collision_group_whitelist: u32,
    collision_group_blacklist: u32,
}

impl PhysicsBodyComponentPrototypeBox {
    pub fn new(
        size: RectSize,
        mass: f32,
        collision_group_membership: u32,
        collision_group_whitelist: u32,
        collision_group_blacklist: u32,
    ) -> Self {
        PhysicsBodyComponentPrototypeBox {
            size,
            mass,
            collision_group_membership,
            collision_group_whitelist,
            collision_group_blacklist,
        }
    }
}

impl Default for PhysicsBodyComponentPrototypeBox {
    fn default() -> Self {
        PhysicsBodyComponentPrototypeBox {
            size: transform::default_scale(),
            mass: 0.0,
            collision_group_membership: 0,
            collision_group_whitelist: 0,
            collision_group_blacklist: 0,
        }
    }
}

impl ComponentPrototype for PhysicsBodyComponentPrototypeBox {
    type Factory = PhysicsBodyComponentFactory;
}

impl FrameworkComponentPrototype for PhysicsBodyComponentPrototypeBox {
    fn component_type() -> std::any::TypeId {
        std::any::TypeId::of::<PhysicsBodyComponent>()
    }
}

#[cfg(feature = "editor")]
impl SelectableComponentPrototype<Self> for PhysicsBodyComponentPrototypeBox {
    fn create_selection_shape(
        framework_entity: &FrameworkEntityPrototypeInner,
        data: &Self,
    ) -> (
        ncollide::math::Isometry<f32>,
        ncollide::shape::ShapeHandle<f32>,
    ) {
        let mut scale = transform::default_scale();
        if let Some(transform) = framework_entity.find_component_prototype::<TransformComponentPrototype>() {
            scale = transform.data().scale();
        }

        let extents = scale.component_mul(&data.size);
        use ncollide::shape::{Cuboid, ShapeHandle};
        (
            ncollide::math::Isometry::<f32>::new(glm::zero(), glm::zero()),
            ShapeHandle::new(Cuboid::new(extents / 2.0)),
        )
    }
}

//
// Circle Prototype
//
#[derive(Clone, Serialize, Deserialize, Inspect)]
pub struct PhysicsBodyComponentPrototypeCircle {
    radius: f32,
    mass: f32,
    collision_group_membership: u32,
    collision_group_whitelist: u32,
    collision_group_blacklist: u32,
}

impl PhysicsBodyComponentPrototypeCircle {
    pub fn new(
        radius: f32,
        mass: f32,
        collision_group_membership: u32,
        collision_group_whitelist: u32,
        collision_group_blacklist: u32,
    ) -> Self {
        PhysicsBodyComponentPrototypeCircle {
            radius,
            mass,
            collision_group_membership,
            collision_group_whitelist,
            collision_group_blacklist,
        }
    }
}

impl Default for PhysicsBodyComponentPrototypeCircle {
    fn default() -> Self {
        PhysicsBodyComponentPrototypeCircle {
            radius: 10.0,
            mass: 0.0,
            collision_group_membership: 0,
            collision_group_whitelist: 0,
            collision_group_blacklist: 0,
        }
    }
}

impl ComponentPrototype for PhysicsBodyComponentPrototypeCircle {
    type Factory = PhysicsBodyComponentFactory;
}

impl FrameworkComponentPrototype for PhysicsBodyComponentPrototypeCircle {
    fn component_type() -> std::any::TypeId {
        std::any::TypeId::of::<PhysicsBodyComponent>()
    }
}

#[cfg(feature = "editor")]
impl SelectableComponentPrototype<Self> for PhysicsBodyComponentPrototypeCircle {
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

        use ncollide::shape::{Ball, ShapeHandle};
        (
            ncollide::math::Isometry::<f32>::new(glm::zero(), glm::zero()),
            ShapeHandle::new(Ball::new((data.radius * scale).max(std::f32::MIN_POSITIVE))),
        )
    }
}

//
// Factory for PhysicsBody components
//
enum QueuedPhysicsBodyPrototypes {
    Custom(PhysicsBodyComponentPrototypeCustom),
    Box(PhysicsBodyComponentPrototypeBox),
    Circle(PhysicsBodyComponentPrototypeCircle),
}

pub struct PhysicsBodyComponentFactory {
    prototypes: VecDeque<(EntityHandle, QueuedPhysicsBodyPrototypes)>,
}

impl PhysicsBodyComponentFactory {
    pub fn new() -> Self {
        PhysicsBodyComponentFactory {
            prototypes: VecDeque::new(),
        }
    }
}

impl ComponentFactory<PhysicsBodyComponentPrototypeCustom> for PhysicsBodyComponentFactory {
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &PhysicsBodyComponentPrototypeCustom,
    ) {
        self.prototypes.push_back((
            entity_handle.clone(),
            QueuedPhysicsBodyPrototypes::Custom(prototype.clone()),
        ));
    }
}

impl ComponentFactory<PhysicsBodyComponentPrototypeBox> for PhysicsBodyComponentFactory {
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &PhysicsBodyComponentPrototypeBox,
    ) {
        self.prototypes.push_back((
            entity_handle.clone(),
            QueuedPhysicsBodyPrototypes::Box(prototype.clone()),
        ));
    }
}

impl ComponentFactory<PhysicsBodyComponentPrototypeCircle> for PhysicsBodyComponentFactory {
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &PhysicsBodyComponentPrototypeCircle,
    ) {
        self.prototypes.push_back((
            entity_handle.clone(),
            QueuedPhysicsBodyPrototypes::Circle(prototype.clone()),
        ));
    }
}

fn create_collision_groups(
    membership: u32,
    whitelist: u32,
    blacklist: u32,
) -> ncollide::world::CollisionGroups {
    // Start with an empty group. (If we don't specify membership, it will default to being in all groups)
    let mut groups = ncollide::world::CollisionGroups::new()
        .with_membership(&[])
        .with_whitelist(&[])
        .with_blacklist(&[]);

    for i in 0..ncollide::world::CollisionGroups::max_group_id() as u32 {
        groups.modify_membership(i as usize, membership & (1 << i) != 0);
        groups.modify_whitelist(i as usize, whitelist & (1 << i) != 0);
        groups.modify_blacklist(i as usize, blacklist & (1 << i) != 0);
    }

    groups
}

impl ComponentCreateQueueFlushListener for PhysicsBodyComponentFactory {
    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        //TODO: Either need two-phase entity construction or deterministic construct order.
        let transform = resource_map.fetch::<<TransformComponent as Component>::Storage>();

        let mut physics = resource_map.fetch_mut::<crate::resources::PhysicsManager>();
        let mut storage = resource_map.fetch_mut::<<PhysicsBodyComponent as Component>::Storage>();
        for (entity_handle, data) in self.prototypes.drain(..) {
            if let Some(entity) = entity_set.get_entity_ref(&entity_handle) {
                let (center, scale, rotation) : (transform::Position, transform::Scale, transform::Rotation) =
                    if let Some(p) = entity.get_component::<TransformComponent>(&*transform) {
                        (p.position(), p.scale(), p.rotation())
                    } else {
                        (transform::default_position(), transform::default_scale(), transform::default_rotation())
                    };

                //TODO: There is a silly amount of duplicated code in here
                match data {
                    QueuedPhysicsBodyPrototypes::Box(data) => {
                        use ncollide::shape::{Cuboid, ShapeHandle};
                        use nphysics::material::{BasicMaterial, MaterialHandle};

                        let mut x_half_extent = (scale.x * data.size.x) / 2.0;
                        if x_half_extent < std::f32::MIN_POSITIVE {
                            warn!("Tried to create a box with with <=0 x half-extent");
                            x_half_extent = std::f32::MIN_POSITIVE;
                        }

                        let mut y_half_extent = (scale.y * data.size.y) / 2.0;
                        if y_half_extent < std::f32::MIN_POSITIVE {
                            warn!("Tried to create a box with with <=0 y half-extent");
                            y_half_extent = std::f32::MIN_POSITIVE;
                        }

                        #[cfg(feature = "dim2")]
                        let half_extents = glm::vec2(x_half_extent, y_half_extent);

                        #[cfg(feature = "dim3")]
                        let half_extents = {
                            let mut z_half_extent = (scale.y * data.size.y) / 2.0;
                            if z_half_extent < std::f32::MIN_POSITIVE {
                                warn!("Tried to create a box with with <=0 y half-extent");
                                z_half_extent = std::f32::MIN_POSITIVE;
                            }

                            glm::vec3(x_half_extent, y_half_extent, z_half_extent)
                        };

                        let shape = ShapeHandle::new(Cuboid::new(half_extents));

                        let collider_desc = ColliderDesc::new(shape)
                            .material(MaterialHandle::new(BasicMaterial::new(0.0, 0.3)))
                            .collision_groups(create_collision_groups(
                                data.collision_group_membership,
                                data.collision_group_whitelist,
                                data.collision_group_blacklist,
                            ));

                        #[cfg(feature = "dim2")]
                        let isometry = ncollide::math::Isometry::new(center, rotation);

                        #[cfg(feature = "dim3")]
                        let isometry = ncollide::math::Isometry::from_parts(center.into(), UnitQuaternion::from_quaternion(rotation));

                        let body_desc = RigidBodyDesc::new()
                            .position(isometry)
                            .collider(&collider_desc);

                        #[cfg(feature = "dim2")]
                        let body_desc = body_desc.kinematic_rotation(false);
                        #[cfg(feature = "dim3")]
                        let body_desc = body_desc.kinematic_rotations(nphysics::math::Vector::repeat(false));

                        let body = physics.world_mut().add_body(&body_desc);
                        entity
                            .add_component(&mut *storage, PhysicsBodyComponent::new(body.handle()))
                            .unwrap();
                    }

                    QueuedPhysicsBodyPrototypes::Circle(data) => {
                        use ncollide::shape::{Ball, ShapeHandle};
                        use nphysics::material::{BasicMaterial, MaterialHandle};

                        let mut radius = data.radius * f32::max(scale.x, scale.y);

                        if radius < std::f32::MIN_POSITIVE {
                            warn!("Tried to create a circle with <=0 radius");
                            radius = std::f32::MIN_POSITIVE;
                        }

                        let shape =
                            ShapeHandle::new(Ball::new(radius));

                        let collider_desc = ColliderDesc::new(shape)
                            .material(MaterialHandle::new(BasicMaterial::new(0.0, 0.3)))
                            .collision_groups(create_collision_groups(
                                data.collision_group_membership,
                                data.collision_group_whitelist,
                                data.collision_group_blacklist,
                            ));

                        #[cfg(feature = "dim2")]
                        let isometry = ncollide::math::Isometry::new(center, rotation);

                        #[cfg(feature = "dim3")]
                        let isometry = ncollide::math::Isometry::from_parts(center.into(), UnitQuaternion::from_quaternion(rotation));

                        let body_desc = RigidBodyDesc::new()
                            .position(isometry)
                            .mass(data.mass)
                            .collider(&collider_desc);

                        #[cfg(feature = "dim2")]
                        let body_desc = body_desc.kinematic_rotation(false);
                        #[cfg(feature = "dim3")]
                        let body_desc = body_desc.kinematic_rotations(nphysics::math::Vector::repeat(false));

                        let body = physics.world_mut().add_body(&body_desc);
                        entity
                            .add_component(&mut *storage, PhysicsBodyComponent::new(body.handle()))
                            .unwrap();
                    }

                    QueuedPhysicsBodyPrototypes::Custom(data) => {
                        #[cfg(feature = "dim2")]
                        let isometry = ncollide::math::Isometry::new(center, rotation);

                        #[cfg(feature = "dim3")]
                        let isometry = ncollide::math::Isometry::from_parts(center.into(), UnitQuaternion::from_quaternion(rotation));

                        let body_desc = data.desc.clone_rigid_body_desc()
                            .position(isometry);

                        let body = physics.world_mut().add_body(&body_desc);
                        entity
                            .add_component(&mut *storage, PhysicsBodyComponent::new(body.handle()))
                            .unwrap();
                    }
                }
            }
        }
    }
}
