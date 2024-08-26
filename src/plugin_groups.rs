use crate::{
    AssetLoaderPlugin, CameraPlugin, DebugPlugin, GridPlugin, InputsPlugin, MovementPlugin, OrePlugin, PlayerPlugin,
    SchedulePlugin, StatePlugin, StructuresPlugin,
};
use bevy::app::{PluginGroup, PluginGroupBuilder};

/// A group of plugins that has loading assets involved
pub struct LoadersPlugins;
impl PluginGroup for LoadersPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(StatePlugin).add(SchedulePlugin).add(AssetLoaderPlugin)
    }
}

pub struct GamePlugins {
    pub debug_enable: bool,
}
impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(GridPlugin { debug_enable: self.debug_enable })
            .add(InputsPlugin)
            .add(PlayerPlugin)
            .add(MovementPlugin)
            .add(StructuresPlugin { debug_enable: self.debug_enable })
            .add(OrePlugin)
    }
}

pub struct UtilityPlugins {
    pub debug_enable: bool,
}
impl PluginGroup for UtilityPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(DebugPlugin { enable: self.debug_enable }).add(CameraPlugin)
    }
}
