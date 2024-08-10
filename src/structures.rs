use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use crate::state::GameState;
use bevy::color::palettes::css::*;
use bevy::math::Vec3;
use bevy::sprite::MaterialMesh2dBundle;
use crate::grid::{Grid};
use crate::player::{InputAction, Player};
use bevy::color::palettes::css::*;

use crate::asset_loader::{AssetBlob, AssetStore, StructuresData};


#[derive(Default)]
pub struct StructuresPlugin {
    pub debug_enable: bool,
}

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::BuildingStructures), setup_structures_from_file)
        .add_systems(Update, detect_player_in_command_center.run_if(in_state(GameState::InGame)));

        if self.debug_enable {
            app.add_systems(Update, (debug_draw_structure_grid,debug_draw_player_rect_grid_in_structure).chain().run_if(in_state(GameState::InGame)));
        }
    }
}

#[derive(Debug)]
struct Module {
    inner_grid_pos: (i32, i32),
    module_type: ModuleType,
    entity_controlling: Option<Entity>,
}

#[derive(Debug)]
enum ModuleType {
    Engine,
    CommandCenter,
    LivingQuarters,
    Storage,
    Walkable
}

#[derive(Bundle)]
struct StructureBundle {
    structure: Structure,
    transform: Transform,
}

#[derive(Component, Debug)]
pub struct Structure {
    grid: Grid,
    modules: Vec<Module>,
}

impl Structure {
    pub fn new() -> Self {
        Structure {
            grid: Default::default(),
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Module) {
        self.modules.push(module);
    }

    pub fn get_modules(&self) -> &Vec<Module> {
        &self.modules
    }
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


        for structure_data in structures.structures {

            let mut structure_component = Structure::new();


            structure_component.grid = Grid::new(
                structure_data[0].len() as u32, // Width of the structure
                structure_data.len() as u32,    // Height of the structure
                50.0,                           // Cell size
            );

            for (y, row) in structure_data.iter().enumerate() {
                for (x, cell) in row.chars().enumerate() {
                    // Match the character to determine the type of module to spawn
                    let module_type = match cell {
                        'E' => {
                            let engine_module = Module {
                                inner_grid_pos: (x as i32, y as i32),
                                module_type: ModuleType::Engine,
                                entity_controlling: None,
                            };
                            structure_component.add_module(engine_module);
                        },
                        'C' => {
                            let command_center_module = Module {
                                inner_grid_pos: (x as i32, y as i32),
                                module_type: ModuleType::CommandCenter,
                                entity_controlling: None,
                            };
                            structure_component.add_module(command_center_module);
                        },
                        'W' => {
                            let walkable_module = Module {
                                inner_grid_pos: (x as i32, y as i32),
                                module_type: ModuleType::Walkable,
                                entity_controlling: None,
                            };
                            structure_component.add_module(walkable_module);
                        },
                        _ => continue, // Skip characters that don't correspond to a module
                    };


                    structure_component.grid.insert(
                        x as i32,
                        y as i32,
                    );

                }
            }

            commands.spawn(StructureBundle {
                structure: structure_component,
                transform: Transform::from_translation(Vec3::new(-500.0, 100.0, 1.0)),
            });
        }
        next_state.set(GameState::InGame);
    } else {
        panic!("Failed to load structures asset");
    }
}

fn detect_player_in_command_center(
    mut event_reader: EventReader<InputAction>,
    mut structure_query: Query<(&mut Structure, &Transform)>,
    player_query: Query<(Entity, &Transform), With<Player>>,
) {

    //loop for player pos
    for (player_entity, player_transform) in &player_query {
        for (mut structure, structure_transform) in &mut structure_query {
            let player_world_pos = player_transform.translation - structure_transform.translation;
            let player_grid_pos = structure.grid.world_to_grid(player_world_pos);

            // Check if the player is within the grid boundaries
            if player_grid_pos.0 >= 0 && player_grid_pos.0 < structure.grid.width as i32 &&
                player_grid_pos.1 >= 0 && player_grid_pos.1 < structure.grid.height as i32 {

                // Check if the player is in a Command Center and if so, check if the player is already controlling it
                if let Some(command_center_module) = structure.modules.iter_mut().find(|module| {

                    matches!(module.module_type, ModuleType::CommandCenter) &&
                        module.inner_grid_pos == player_grid_pos &&
                        module.entity_controlling.is_none() // Also checks if no entity is currently controlling it
                }) {
                    // Player can control the Command Center if wanted.
                    for event in event_reader.read() {

                        if let InputAction::SpacePressed = event {
                            // Set the entity controlling the Command Center
                            command_center_module.entity_controlling = Some(player_entity);
                            debug!("Player is now controlling the Command Center.");
                        }

                    }
                } else {
                    // debug!("Player is not in a Command Center or the Command Center is already being controlled.");
                }


            }
        }
    }

}

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
    structure_query: Query<(&Transform, &Structure)>,
) {
    for (transform, structure) in &structure_query {

        // loop on structure modules
        for module in structure.get_modules() {
            let world_pos = structure.grid.grid_to_world(module.inner_grid_pos) + transform.translation;
            let square_size = structure.grid.cell_size * 0.90; // Adjust this value to control the size of the square

            let color = match module.module_type {
                ModuleType::Engine => RED,
                ModuleType::CommandCenter => BLUE,
                ModuleType::LivingQuarters => YELLOW,
                ModuleType::Storage => PURPLE,
                ModuleType::Walkable => GREY,
            };

            gizmos.rect_2d(
                Vec2::new(world_pos.x, world_pos.y),
                0.0,
                Vec2::splat(square_size),
                color,
            );
        }
    }
}


fn debug_draw_player_rect_grid_in_structure(
    mut gizmos: Gizmos,
    structure_query: Query<(&Transform, &Structure)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_color = GREEN;

    for (structure_transform, structure) in &structure_query {

        for player_transform in &player_query {
            let player_world_pos = player_transform.translation - structure_transform.translation;
            let player_grid_pos = structure.grid.world_to_grid(player_world_pos);
            let square_size = structure.grid.cell_size * 0.90; // Adjust this value to control the size of the square

            // Check if the player is within the grid boundaries
            if player_grid_pos.0 >= 0 && player_grid_pos.0 < structure.grid.width as i32 &&
                player_grid_pos.1 >= 0 && player_grid_pos.1 < structure.grid.height as i32 {
                let player_world_pos = structure.grid.grid_to_world(player_grid_pos) + structure_transform.translation;
                gizmos.rect_2d(
                    Vec2::new(player_world_pos.x, player_world_pos.y),
                    0.0,
                    Vec2::splat(square_size),
                    player_color,
                );
            }
        }

    }
}