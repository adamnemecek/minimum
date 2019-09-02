
extern crate nalgebra_glm as glm;

#[macro_use]
extern crate log;

#[macro_use]
extern crate named_type_derive;

#[macro_use]
extern crate imgui_inspect_derive;

#[macro_use]
extern crate mopa;

mod clone_component;
pub use clone_component::CloneComponentFactory;
pub use clone_component::CloneComponentPrototype;

pub mod components;
pub mod resources;

pub mod inspect;

pub mod persist;

pub mod select;

mod prototype;
pub use prototype::FrameworkComponentPrototype;
pub use prototype::FrameworkEntityPrototype;
pub use prototype::FrameworkEntityPersistencePolicy;

#[derive(Copy, Clone, PartialEq, strum_macros::EnumCount, Debug)]
pub enum PlayMode {
    // Represents the game being frozen for debug purposes
    System,

    // Represents the game being puased by the user (actual meaning of this is game-specific)
    Paused,

    // Normal simulation is running
    Playing,
}

//PLAYMODE_COUNT exists due to strum_macros::EnumCount
const PLAY_MODE_COUNT: usize = PLAYMODE_COUNT;

pub mod context_flags {
    // For pause status. Flags will be set based on if the game is in a certain playmode
    pub const PLAYMODE_SYSTEM: usize = 1;
    pub const PLAYMODE_PAUSED: usize = 2;
    pub const PLAYMODE_PLAYING: usize = 4;

    // For multiplayer games:
    // - Dedicated Server will only run Net_Server
    // - Pure client will only have Net_Client
    // - "Listen" client will have both
    // - Singleplayer will have both
    pub const AUTHORITY_SERVER: usize = 8;
    pub const AUTHORITY_CLIENT: usize = 16;
}