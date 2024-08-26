pub mod asset_loader;
pub mod camera;
pub mod debug;
pub mod grid;
pub mod inputs;
pub mod modules;
pub mod movement;
pub mod ore;
pub mod player;
pub mod plugin_groups;
pub mod prelude;
pub mod schedule;
pub mod state;
pub mod structures;
pub mod structures_combat;
pub mod utils;

pub use asset_loader::AssetLoaderPlugin;
pub use camera::CameraPlugin;
pub use debug::DebugPlugin;
pub use grid::GridPlugin;
pub use inputs::InputsPlugin;
pub use movement::MovementPlugin;
pub use ore::OrePlugin;
pub use player::PlayerPlugin;
pub use schedule::SchedulePlugin;
pub use state::StatePlugin;
pub use structures::StructuresPlugin;

pub const UNIT_SCALE: f32 = 1.0; // 1 pixels = 1 meter
