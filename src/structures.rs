use avian2d::prelude::*;
use bevy::app::{App, Plugin, Update};
use bevy::color::palettes::css::*;
use bevy::math::Vec3;
use bevy::prelude::*;

use crate::asset_loader::{AssetBlob, AssetStore, StructuresData};
use crate::grid::Grid;
use crate::modules::{spawn_module, Module, ModuleType};
use crate::player::{InputAction, Player};
use crate::state::GameState;

#[derive(Default)]
pub struct StructuresPlugin {
    pub debug_enable: bool,
}

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ModuleInteractionEvent>()
            .add_systems(OnEnter(GameState::BuildingStructures), setup_structures_from_file);
        //.add_systems(Update, (control_command_center_system, move_structure_system).run_if(in_state(GameState::InGame)))
        //.add_systems(FixedUpdate, move_structure_system.run_if(in_state(GameState::InGame)));

        if self.debug_enable {
            app.add_systems(
                Update,
                (debug_draw_structure_grid, debug_draw_rects).chain().run_if(in_state(GameState::InGame)),
            );
        }
    }
}

#[derive(Component)]
struct ControlledByPlayer {
    player_entity: Entity,
}

#[derive(Bundle)]
struct StructureBundle {
    rigid_body: RigidBody,
    structure: Structure,
    transform_budle: TransformBundle,
    inherited_visibility: InheritedVisibility,
}

#[derive(Component, Debug)]
pub struct Structure {
    grid: Grid,
}

impl Structure {
    pub fn new() -> Self {
        Structure { grid: Default::default() }
    }

    // Convert the player's world position to a position relative to the structure's grid
    pub fn player_relative_position(&self, player_world_pos: Vec3, structure_transform: &Transform) -> Vec3 {
        player_world_pos - structure_transform.translation
    }

    // Adjust a position for the grid's origin by shifting by half a cell size
    pub fn adjust_for_grid_origin(&self, relative_pos: Vec3) -> Vec3 {
        Vec3::new(
            relative_pos.x + (self.grid.cell_size / 2.0),
            relative_pos.y + (self.grid.cell_size / 2.0),
            relative_pos.z,
        )
    }

    // Function to check if a raw world position is within the grid's bounds
    pub fn is_world_position_within_grid(&self, world_pos: Vec3, structure_transform: &Transform) -> bool {
        // Convert the player's world position to the structure's local grid coordinates
        let relative_pos = world_pos - structure_transform.translation;

        // Adjust for the grid's origin (if necessary)
        let adjusted_pos = Vec3::new(
            relative_pos.x + (self.grid.cell_size / 2.0),
            relative_pos.y + (self.grid.cell_size / 2.0),
            relative_pos.z,
        );

        // Convert the adjusted position to grid coordinates
        let (grid_x, grid_y) = self.grid.world_to_grid(adjusted_pos);

        // Check if these coordinates are within the grid's bounds
        self.is_within_grid_bounds(grid_x, grid_y)
    }

    // Check if the grid coordinates are within the grid's bounds
    pub fn is_within_grid_bounds(&self, grid_x: i32, grid_y: i32) -> bool {
        grid_x >= 0 && grid_x < self.grid.width as i32 && grid_y >= 0 && grid_y < self.grid.height as i32
    }

    // Convert grid coordinates back to world coordinates and apply structure's translation
    pub fn grid_to_world_position(&self, grid_pos: (i32, i32), structure_transform: &Transform) -> Vec3 {
        let world_pos = self.grid.grid_to_world(grid_pos) + structure_transform.translation;
        Vec3::new(world_pos.x - (self.grid.cell_size / 2.0), world_pos.y - (self.grid.cell_size / 2.0), world_pos.z)
    }
}

fn setup_structures_from_file(
    mut commands: Commands,
    asset_store: Res<AssetStore>,
    blob_assets: Res<Assets<AssetBlob>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some(blob) = blob_assets.get(&asset_store.structures_blob) {
        let structures_data: String = String::from_utf8(blob.bytes.clone()).expect("Invalid UTF-8 data");
        let structures: StructuresData =
            serde_json::from_str(&structures_data).expect("Failed to deserialize structures data");

        for structure_data in structures.structures {
            let mut structure_component = Structure::new();

            let grid_width = structure_data[0].len() as f32;
            let grid_height = structure_data.len() as f32;

            let mesh_scale_factor = 0.97; // Adjust this value to reduce the mesh size

            structure_component.grid = Grid::new(
                grid_width as u32,  // Width of the structure
                grid_height as u32, // Height of the structure
                50.0,               // Cell size
            );

            let structure_entity = commands.spawn_empty().id();

            for (y, row) in structure_data.iter().enumerate() {
                for (x, cell) in row.chars().enumerate() {
                    let x_translation = (x as f32 - grid_width / 2.0) * structure_component.grid.cell_size;
                    let y_translation = (grid_height / 2.0 - y as f32) * structure_component.grid.cell_size;

                    // Match the character to determine the type of module to spawn
                    match cell {
                        'E' => {
                            spawn_module(
                                &mut commands,
                                structure_entity,
                                &mut materials,
                                &mut meshes,
                                ModuleType::Engine,
                                Color::from(RED),
                                (x as i32, y as i32),
                                Vec3::new(x_translation, y_translation, 1.0),
                                structure_component.grid.cell_size,
                                mesh_scale_factor,
                            );
                        }
                        'W' => {
                            spawn_module(
                                &mut commands,
                                structure_entity,
                                &mut materials,
                                &mut meshes,
                                ModuleType::Wall,
                                Color::from(BLUE),
                                (x as i32, y as i32),
                                Vec3::new(x_translation, y_translation, 1.0),
                                structure_component.grid.cell_size,
                                mesh_scale_factor,
                            );
                        }
                        _ => continue, // Skip characters that don't correspond to a module
                    };

                    structure_component.grid.insert(x as i32, y as i32);
                }
            }

            // Insert the structure bundle
            commands.entity(structure_entity).insert(StructureBundle {
                rigid_body: RigidBody::Static,
                structure: structure_component,
                transform_budle: TransformBundle {
                    local: Transform::from_translation(Vec3::new(500.0, 200.0, 1.0)),
                    ..default()
                },
                inherited_visibility: InheritedVisibility::default(),
            });
        }
        next_state.set(GameState::InGame);
    } else {
        panic!("Failed to load structures asset");
    }
}

#[derive(Event)]
pub enum ModuleInteractionEvent {
    TakeControl { player_entity: Entity, structure_entity: Entity },
    ReleaseControl { player_entity: Entity, structure_entity: Entity },
}

fn control_command_center_system(
    mut event_reader: EventReader<InputAction>,
    mut event_writer: EventWriter<ModuleInteractionEvent>,
    mut structure_query: Query<(Entity, &mut Structure, &Transform)>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut command: Commands,
) {

    // //loop for player pos
    // for (player_entity, player_transform) in &player_query {
    //     for (structure_entity, mut structure, structure_transform) in &mut structure_query {
    //         let player_world_pos = player_transform.translation - structure_transform.translation;
    //         let player_grid_pos = structure.grid.world_to_grid(player_world_pos);
    //
    //         // Check if the player is within the grid boundaries
    //         if player_grid_pos.0 >= 0 && player_grid_pos.0 < structure.grid.width as i32 &&
    //             player_grid_pos.1 >= 0 && player_grid_pos.1 < structure.grid.height as i32 {
    //
    //             // Check if the player is in a Command Center and if so, check if the player is already controlling it
    //             if let Some(command_center_module) = structure.modules.iter_mut().find(|module| {
    //
    //                 matches!(module.module_type, ModuleType::CommandCenter) &&
    //                     matches!(module.module_type, ModuleType::CommandCenter) && // Checking if the module is a Command Center
    //                     module.inner_grid_pos == player_grid_pos // Checking if the player is in the Command Center
    //             }) {
    //                 // Player can control or release the Command Center by pressing the spacebar.
    //                 for event in event_reader.read() {
    //                     if let InputAction::SpacePressed = event {
    //                         if command_center_module.entity_controlling.is_none() {
    //                             // Take control if no one is controlling it
    //                             command_center_module.entity_controlling = Some(player_entity);
    //                             debug!("Player is now controlling the Command Center.");
    //
    //
    //                             // lets insert the PlayerControlled component to the structure
    //                             command.entity(structure_entity).insert(ControlledByPlayer {
    //                                 player_entity,
    //                             });
    //
    //                             event_writer.send(ModuleInteractionEvent::TakeControl {
    //                                 player_entity,
    //                                 structure_entity,
    //                             });
    //                         } else if command_center_module.entity_controlling == Some(player_entity) {
    //                             // Release control if the player is already controlling it
    //                             command_center_module.entity_controlling = None;
    //                             debug!("Player has released control of the Command Center.");
    //
    //                             // lets remove the PlayerControlled component from the structure
    //                             command.entity(structure_entity).remove::<ControlledByPlayer>();
    //
    //                             // Emit an event for releasing control
    //                             event_writer.send(ModuleInteractionEvent::ReleaseControl {
    //                                 player_entity,
    //                                 structure_entity,
    //                             });
    //                         }
    //                     }
    //                 }
    //             } else {
    //                 // debug!("Player is not in a Command Center or the is not the module.");
    //             }
    //         }
    //     }
    // }
}

fn move_structure_system(
    mut controlled_structures_query: Query<(Entity, &mut LinearVelocity, &ControlledByPlayer), With<Structure>>,
    player_query: Query<(Entity, &LinearVelocity), (With<Player>, Without<Structure>)>,
) {

    // // Loop through all structures that are controlled by the player
    // for (structure_entity, mut structure_velocity, controlled_by) in &mut controlled_structures_query {
    //     if let Ok((player_entity, player_velocity)) = player_query.get(controlled_by.player_entity) {
    //         // Set the structure's velocity to match the player's velocity
    //         *structure_velocity = *player_velocity;
    //     }
    // }
}

fn debug_draw_structure_grid(
    mut gizmos: Gizmos,
    structures_query: Query<(&Transform, &Structure)>,
    mut parent_query: Query<(Entity, &Children), With<Structure>>,
    mut child_query: Query<&mut Module>,
) {
    for (transform, structure) in &structures_query {
        let world_pos = transform.translation;
        let grid = &structure.grid;

        // Draw the grid
        gizmos
            .grid_2d(
                Vec2::new(world_pos.x - grid.cell_size / 2.0, world_pos.y + grid.cell_size / 2.0),
                0.0,
                UVec2::new(grid.width, grid.height),
                Vec2::splat(grid.cell_size),
                Color::from(GREY),
            )
            .outer_edges();
    }

    /* for (parent, childrens) in &mut parent_query {
        // To iterate through the entities children, just treat the Children component as a Vec
        // Alternatively, you could query entities that have a Parent component
        for child in childrens {
            if let Ok(module) = child_query.get_mut(*child) {
                let world_pos = Vec2::new(module.inner_grid_pos.0 as f32, module.inner_grid_pos.1 as f32);
                let square_size = 50.0 * 0.90; // Adjust this value to control the size of the square

                gizmos.rect_2d(world_pos, 0.0, Vec2::splat(square_size), GREY);
            }
        }
    } */

    /* for (transform, structure) in &structure_query {
        // loop on structure modules
        for engine in structure_modules {
            let world_pos =
                structure.grid.grid_to_world(engine.inner_grid_pos) + transform.translation;
            let square_size = structure.grid.cell_size * 0.90; // Adjust this value to control the size of the square

            gizmos.rect_2d(
                Vec2::new(world_pos.x, world_pos.y),
                0.0,
                Vec2::splat(square_size),
                GREEN,
            );
        }
    } */
}

fn debug_draw_rects(
    mut gizmos: Gizmos,
    query: Query<&Transform, With<Player>>,
    structures_query: Query<(&Transform, &Structure)>,
) {
    for player_transform in &query {
        for (structure_transform, structure) in &structures_query {
            let grid = &structure.grid;
            let square_size = grid.cell_size * 0.95; // Adjust this value to control the size of the square

            // Get the player's position relative to the structure
            let player_relative_pos =
                structure.player_relative_position(player_transform.translation, structure_transform);

            // Adjust for the grid's origin
            let adjusted_player_pos = structure.adjust_for_grid_origin(player_relative_pos);

            // Convert the adjusted position to grid coordinates
            let (grid_x, grid_y) = grid.world_to_grid(adjusted_player_pos);

            // Check if the player's grid coordinates are within the grid's bounds
            if structure.is_within_grid_bounds(grid_x, grid_y) {
                // Player is inside the structure's grid

                // Get the world position for drawing
                let world_pos = structure.grid_to_world_position((grid_x, grid_y), structure_transform);

                // Draw a green rectangle at the player's current grid position within the structure's grid
                gizmos.rect_2d(Vec2::new(world_pos.x, world_pos.y), 0.0, Vec2::splat(square_size), GREEN);
            }
        }
    }
}
