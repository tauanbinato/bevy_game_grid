use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use crate::state::GameState;
use bevy::color::palettes::css::*;
use bevy::math::Vec3;
use bevy::sprite::MaterialMesh2dBundle;
use crate::grid::{Grid, StructuresGrid};
use crate::player::{InputAction, Player};
use bevy::color::palettes::css::*;

use crate::asset_loader::{AssetBlob, AssetStore, StructuresData};


#[derive(Default)]
pub struct StructuresPlugin {
    pub debug_enable: bool,
}

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::BuildingStructures), setup_structures_from_file);
        //.add_systems(Update, detect_player_in_command_center.run_if(in_state(GameState::InGame)));

        if self.debug_enable {
            app.add_systems(Update, (debug_draw_structure_grid,).chain().run_if(in_state(GameState::InGame)));
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum ModuleType {
    #[default]
    Engine,
    CommandCenter { controlling_entity: Option<Entity> },
    LivingQuarters,
    Storage,
    Weapon,
}

#[derive(Default, Component, Clone)]
pub struct Module {
    pub module_type: ModuleType,
}

#[derive(Bundle)]
pub struct ModuleBundle {
    module: Module,
}

#[derive(Component)]
pub struct Structure;

#[derive(Bundle)]
struct StructureBundle {
    marker: Structure,
    command_center: ModuleBundle,
}

fn setup_structures_from_file(
    mut commands: Commands,
    asset_store: Res<AssetStore>,
    blob_assets: Res<Assets<AssetBlob>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(blob) = blob_assets.get(&asset_store.structures_blob) {
        let structures_data: String = String::from_utf8(blob.bytes.clone()).expect("Invalid UTF-8 data");
        let structures: StructuresData = serde_json::from_str(&structures_data).expect("Failed to deserialize structures data");

        let mut structures_grid = StructuresGrid::default();

        for structure_data in structures.structures {
            let mut grid = Grid::new(structure_data[0].len() as u32, structure_data.len() as u32, 50.0);

            for (y, row) in structure_data.iter().enumerate() {
                for (x, cell) in row.chars().enumerate() {
                    let module_type = match cell {
                        'E' => ModuleType::Engine,
                        'C' => ModuleType::CommandCenter { controlling_entity: None },
                        'L' => ModuleType::LivingQuarters,
                        'S' => ModuleType::Storage,
                        'W' => ModuleType::Weapon,
                        _ => continue,
                    };

                    grid.insert_new(x as i32, y as i32, module_type.clone());

                    let module_bundle = ModuleBundle {
                        module: Module { module_type }
                    };

                    commands.spawn(module_bundle);
                }
            }

            structures_grid.grids.push(grid);
        }
        commands.insert_resource(structures_grid);
        next_state.set(GameState::InGame);
    } else {
        panic!("Failed to load structures asset");
    }
}

// fn detect_player_in_command_center(
//     mut event_reader: EventReader<InputAction>,
//     structure_query: Query<&mut Structure>,
//     player_query: Query<(Entity, &Transform), With<Player>>,
// ) {
//     for event in event_reader.read() {
//
//         if let InputAction::SpacePressed = event {
//             for structure in &structure_query {
//                 let structure_grid = &structure.grid;
//                 let universe_pos = structure.universe_pos.translation;
//
//                 for (player_entity, player_transform) in &player_query {
//                     let player_world_pos = player_transform.translation - universe_pos;
//                     let player_grid_pos = structure_grid.world_to_grid(player_world_pos);
//
//                     if player_grid_pos.0 >= 0 && player_grid_pos.0 < structure_grid.width as i32 &&
//                         player_grid_pos.1 >= 0 && player_grid_pos.1 < structure_grid.height as i32 {
//                         if let Some(module) = structure_grid.get(player_grid_pos.0, player_grid_pos.1) {
//                             if let Some(module) = &module.data {
//                                 if let ModuleType::CommandCenter { mut controlling_entity } = module.module_type {
//
//                                     // Lets check if player is already controlling the Command Center
//                                     if controlling_entity.is_some() {
//
//                                         // Patter matching to get the entity that is controlling the Command Center
//                                         if let Some(entity) = controlling_entity {
//                                             debug!("Entity Controlling is {:?}", entity);
//                                             if entity == player_entity {
//                                                 controlling_entity = None;
//                                                 debug!("Player is no longer controlling the Command Center!");
//                                             } else {
//                                                 controlling_entity = Some(player_entity);
//                                                 debug!("Player is now controlling the Command Center!");
//                                             }
//                                         }
//                                     }
//                                     else {
//                                         controlling_entity = Some(player_entity);
//                                         debug!("No Player was controlling now {:?} is controlling the command center", player_entity);
//                                     }
//
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// fn move_structure_system(
//     mut structure_query: Query<&mut Structure>,
//     mut input_reader: EventReader<InputAction>,
//     player_query: Query<Entity, With<Player>>,
//     time: Res<Time>,
// ) {
//     let delta_time = time.delta_seconds();
//
//     for event in input_reader.read() {
//         if let InputAction::Move(direction) = event {
//             for mut structure in &mut structure_query {
//                 for player_entity in &player_query {
//                     if structure.grid.cells.values().any(|cell| {
//                         if let Some(module) = &cell.data {
//                             if let ModuleType::CommandCenter { controlling_entity } = module.module_type {
//                                 controlling_entity == Some(player_entity)
//                             } else {
//                                 false
//                             }
//                         } else {
//                             false
//                         }
//                     }) {
//                         //structure.universe_pos.translation += direction * 50.0 * delta_time;
//                     }
//                 }
//             }
//         }
//     }
// }

fn debug_draw_structure_grid(
    mut gizmos: Gizmos,
    structures_grid: Res<StructuresGrid>,
    structure_query: Query<(&Transform, &Structure)>,
) {
    for (transform, _structure) in &structure_query {
        for grid in &structures_grid.grids {
            for ((x, y), cell) in &grid.cells {
                let world_pos = grid.grid_to_world((*x, *y)) + transform.translation;
                let color = match &cell.data {
                    Some(module) => match module {
                        ModuleType::Engine => RED,
                        ModuleType::CommandCenter { .. } => BLUE,
                        ModuleType::LivingQuarters => GREEN,
                        ModuleType::Storage => YELLOW,
                        ModuleType::Weapon => PURPLE,
                    },
                    None => GREY,
                };

                gizmos.rect_2d(
                    Vec2::new(world_pos.x, world_pos.y),
                    0.0,
                    Vec2::splat(grid.cell_size * 0.95),
                    color,
                );
            }
        }
    }
}


// New system to debug draw player position within the structure's grid
// fn debug_draw_player_in_structure(
//     mut gizmos: Gizmos,
//     structure_query: Query<&Structure>,
//     player_query: Query<&Transform, With<Player>>,
// ) {
//     let player_color = GREEN;
//
//
//     for structure in &structure_query {
//         let grid = &structure.grid;
//         let universe_pos = structure.universe_pos.translation;
//
//         // Draw player position within the structure's grid
//         for player_transform in &player_query {
//             let player_world_pos = player_transform.translation - universe_pos;
//             let player_grid_pos = grid.world_to_grid(player_world_pos);
//             let square_size = grid.cell_size * 0.90; // Adjust this value to control the size of the square
//
//             // Check if the player is within the grid boundaries
//             if player_grid_pos.0 >= 0 && player_grid_pos.0 < grid.width as i32 &&
//                 player_grid_pos.1 >= 0 && player_grid_pos.1 < grid.height as i32 {
//                 let player_world_pos = grid.grid_to_world(player_grid_pos) + universe_pos;
//                 gizmos.rect_2d(
//                     Vec2::new(player_world_pos.x, player_world_pos.y),
//                     0.0,
//                     Vec2::splat(square_size),
//                     player_color,
//                 );
//             }
//         }
//     }
// }