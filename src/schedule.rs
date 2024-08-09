use bevy::prelude::*;

use crate::state::GameState;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum InGameSet {
    UserInput,
    EntityUpdates,
    CollisionDetection,
    DespawnEntities,
    SpawnEntities,
    Debug
}
//
// #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
// pub enum InLoadGridSet {
//     AssetLoading,
//     GridSetup,
// }

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                InGameSet::SpawnEntities,
                InGameSet::DespawnEntities,
                // Flush commands (i.e. `apply_deferred` runs)
                InGameSet::UserInput,
                InGameSet::EntityUpdates,
                InGameSet::CollisionDetection,
                InGameSet::Debug
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        );

    }
}
