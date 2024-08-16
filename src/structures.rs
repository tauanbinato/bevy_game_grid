use avian2d::prelude::*;
use bevy::app::{App, Plugin, Update};
use bevy::color::palettes::css::*;
use bevy::math::Vec3;
use bevy::prelude::*;

use crate::asset_loader::{AssetBlob, AssetStore, StructuresData};
use crate::grid::Grid;
use crate::inputs::InputAction;
use crate::modules::{spawn_module, Module, ModuleType};
use crate::player::{self, Player, PlayerResource};
use crate::state::GameState;

#[derive(Default)]
pub struct StructuresPlugin {
    pub debug_enable: bool,
}

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ModuleInteractionEvent>()
            .add_event::<StructureInteractionEvent>()
            .add_systems(OnEnter(GameState::BuildingStructures), setup_structures_from_file)
            .add_systems(
                Update,
                (
                    detect_player_inside_structure_system,
                    make_player_child_of_structure_system,
                    control_command_center_system,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(FixedUpdate, move_structure_system.run_if(in_state(GameState::InGame)));

        if self.debug_enable {
            app.add_systems(
                Update,
                (debug_draw_structure_grid, debug_draw_player_inside_structure_rect)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
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
    collider: Collider,
    structure: Structure,
    spatial_bundle: SpatialBundle,
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
    pub fn get_relative_position(&self, some_world_pos: Vec3, structure_transform: &Transform) -> Vec3 {
        some_world_pos - structure_transform.translation
    }

    // Adjust a position for the grid's origin by shifting by half a cell size
    pub fn adjust_for_grid_origin(&self, relative_pos: Vec3) -> Vec3 {
        Vec3::new(
            relative_pos.x + (self.grid.cell_size / 2.0),
            relative_pos.y - (self.grid.cell_size / 2.0),
            relative_pos.z,
        )
    }

    // Convert the player's world position to grid coordinates relative to the structure
    pub fn world_to_grid(&self, global_world_pos: Vec3, structure_transform: &Transform) -> (i32, i32) {
        // Convert the world position to the structure's local space
        let relative_pos = self.get_relative_position(global_world_pos, structure_transform);

        // Adjust for the grid's origin
        let adjusted_pos = self.adjust_for_grid_origin(relative_pos);

        // Convert the relative position to grid coordinates
        self.grid.world_to_grid(adjusted_pos)
    }

    // Function to check if a raw world position is within the grid's bounds
    pub fn is_world_position_within_grid(&self, global_world_pos: Vec3, structure_transform: &Transform) -> bool {
        // Convert the world position to the structure's local space
        let relative_pos = self.get_relative_position(global_world_pos, structure_transform);

        // Adjust for the grid's origin
        let adjusted_pos = self.adjust_for_grid_origin(relative_pos);

        // Convert the adjusted position to grid coordinates
        let (grid_x, grid_y) = self.grid.world_to_grid(adjusted_pos);

        // Check if these coordinates are within the grid's bounds
        self.is_within_grid_bounds(grid_x, grid_y)
    }

    // Check if some grid coordinates are within the grid's bounds
    pub fn is_within_grid_bounds(&self, grid_x: i32, grid_y: i32) -> bool {
        grid_x >= 0 && grid_x < self.grid.width as i32 && grid_y >= 0 && grid_y < self.grid.height as i32
    }

    // Convert grid coordinates back to world coordinates and apply structure's translation
    pub fn grid_to_world_position(&self, grid_pos: (i32, i32), structure_transform: &Transform) -> Vec3 {
        let half_width = self.grid.width as f32 * self.grid.cell_size / 2.0;
        let half_height = self.grid.height as f32 * self.grid.cell_size / 2.0;

        // Adjust the position by the structure's transform and center it within the cell
        Vec3::new(
            grid_pos.0 as f32 * self.grid.cell_size - half_width,
            half_height - grid_pos.1 as f32 * self.grid.cell_size, // Adjusted for top-left origin
            structure_transform.translation.z,                     // Preserve the Z position
        ) + structure_transform.translation
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

            debug!("Grid width: {}, Grid height: {}", grid_width, grid_height);

            let mesh_scale_factor = 0.97; // Adjust this value to reduce the mesh size

            structure_component.grid = Grid::new(
                grid_width as u32,  // Width of the structure
                grid_height as u32, // Height of the structure
                50.0,               // Cell size
            );

            let structure_entity = commands.spawn_empty().id();

            for (y, row) in structure_data.iter().enumerate() {
                for (x, cell) in row.chars().enumerate() {
                    let x_translation = ((x as f32 - (grid_width / 2.0)) * structure_component.grid.cell_size)
                        + (structure_component.grid.cell_size / 2.0);
                    let y_translation = ((grid_height / 2.0) - y as f32) * structure_component.grid.cell_size
                        - (structure_component.grid.cell_size / 2.0);

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
                                false,
                            );
                        }
                        'W' => {
                            spawn_module(
                                &mut commands,
                                structure_entity,
                                &mut materials,
                                &mut meshes,
                                ModuleType::Wall,
                                Color::from(GREY),
                                (x as i32, y as i32),
                                Vec3::new(x_translation, y_translation, 1.0),
                                structure_component.grid.cell_size,
                                mesh_scale_factor,
                                false,
                            );
                        }
                        'C' => {
                            spawn_module(
                                &mut commands,
                                structure_entity,
                                &mut materials,
                                &mut meshes,
                                ModuleType::CommandCenter,
                                Color::from(BLUE),
                                (x as i32, y as i32),
                                Vec3::new(x_translation, y_translation, -1.0),
                                structure_component.grid.cell_size,
                                mesh_scale_factor,
                                true,
                            );

                            debug!("Command Center at ({}, {})", y, x);
                        }
                        _ => continue, // Skip characters that don't correspond to a module
                    };

                    structure_component.grid.insert(y as i32, x as i32);
                }
            }

            // Insert the structure bundle
            commands.entity(structure_entity).insert(StructureBundle {
                rigid_body: RigidBody::Kinematic,
                collider: Collider::rectangle(
                    grid_width * structure_component.grid.cell_size,
                    grid_height * structure_component.grid.cell_size,
                ),
                structure: structure_component,
                spatial_bundle: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(500.0, 200.0, 1.0)),
                    visibility: Visibility::Visible,
                    ..Default::default()
                },
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

#[derive(Event)]
pub enum StructureInteractionEvent {
    PlayerEntered { player_entity: Entity, structure_entity: Entity },
    PlayerExited { player_entity: Entity, structure_entity: Entity },
}

fn detect_player_inside_structure_system(
    mut player_query: Query<(Entity, &GlobalTransform), With<Player>>,
    mut structure_query: Query<(Entity, &Structure, &Transform)>,
    mut event_writer: EventWriter<StructureInteractionEvent>,
    mut player_resource: ResMut<PlayerResource>,
) {
    for (player_entity, player_transform) in &mut player_query {
        for (structure_entity, structure, structure_transform) in &mut structure_query {
            if structure.is_world_position_within_grid(player_transform.translation(), structure_transform) {
                // Emit an event for the player entering the structure
                if player_resource.inside_structure != Some(structure_entity) {
                    player_resource.inside_structure = Some(structure_entity);
                    event_writer.send(StructureInteractionEvent::PlayerEntered { player_entity, structure_entity });
                }
            } else {
                // Emit an event for the player exiting the structure
                if player_resource.inside_structure == Some(structure_entity) {
                    player_resource.inside_structure = None;
                    event_writer.send(StructureInteractionEvent::PlayerExited { player_entity, structure_entity });
                }
            }
        }
    }
}

// TODO: USE OBSERVER INSTEAD OF SYSTEM
fn make_player_child_of_structure_system(
    mut event_reader: EventReader<StructureInteractionEvent>,
    mut command: Commands,
) {
    for event in event_reader.read() {
        match event {
            StructureInteractionEvent::PlayerEntered { player_entity, structure_entity } => {
                command.entity(*player_entity).set_parent_in_place(*structure_entity);
                debug!("Player is now a child of the structure.");
            }
            StructureInteractionEvent::PlayerExited { player_entity, structure_entity: _ } => {
                command.entity(*player_entity).remove_parent_in_place();
                debug!("Player is no longer a child of the structure.");
            }
        }
    }
}

fn control_command_center_system(
    mut event_reader: EventReader<InputAction>,
    mut event_writer: EventWriter<ModuleInteractionEvent>,
    mut player_query: Query<(Entity, &GlobalTransform, &mut LinearVelocity), With<Player>>,
    mut command: Commands,
    mut parent_query: Query<(Entity, &Structure, &Transform, &Children)>,
    mut child_query: Query<&mut Module>,
    mut player_resource: ResMut<PlayerResource>,
) {
    //loop for player pos
    for (player_entity, player_transform, mut player_velocity) in &mut player_query {
        for (structure_entity, structure, structure_transform, children) in &mut parent_query {
            // Convert the adjusted position to grid coordinates
            let (player_grid_x, player_grid_y) =
                structure.world_to_grid(player_transform.translation(), structure_transform);

            // Check if the player's grid coordinates are within the grid's bounds
            if structure.is_within_grid_bounds(player_grid_x, player_grid_y) {
                // Player is inside the structure's grid at this point.
                // Check if the player is in a Command Center and if so, check if the player is already controlling it
                for child in children {
                    if let Ok(mut module) = child_query.get_mut(*child) {
                        if matches!(module.module_type, ModuleType::CommandCenter)
                            && matches!((module.inner_grid_pos.0, module.inner_grid_pos.1), (x, y) if x == player_grid_x && y == player_grid_y)
                        {
                            // Player can control or release the Command Center by pressing the spacebar.
                            for event in event_reader.read() {
                                if let InputAction::SpacePressed = event {
                                    if module.entity_connected.is_none() {
                                        // Take control if no one is controlling it
                                        module.entity_connected = Some(player_entity);
                                        debug!("Player is now controlling the Command Center.");

                                        // lets insert the PlayerControlled component to the structure
                                        command.entity(structure_entity).insert(ControlledByPlayer { player_entity });

                                        // Update the player resource to indicate that the player is controlling a structure
                                        player_resource.is_controlling_structure = true;

                                        // Emit an event for taking control
                                        event_writer.send(ModuleInteractionEvent::TakeControl {
                                            player_entity,
                                            structure_entity,
                                        });
                                    } else if module.entity_connected == Some(player_entity) {
                                        // Release control if the player is already controlling it
                                        module.entity_connected = None;
                                        debug!("Player has released control of the Command Center.");

                                        // lets remove the PlayerControlled component from the structure
                                        command.entity(structure_entity).remove::<ControlledByPlayer>();

                                        // Update the player resource to indicate that the player is not controlling a structure
                                        player_resource.is_controlling_structure = false;

                                        // Emit an event for releasing control
                                        event_writer.send(ModuleInteractionEvent::ReleaseControl {
                                            player_entity,
                                            structure_entity,
                                        });
                                    }
                                }
                            }
                        } else {
                            //debug!("Player is not in a Command Center or the is not the module.");
                        }
                    }
                }
            }
        }
    }
}

fn move_structure_system(
    mut controlled_structure_query: Query<(Entity, &mut LinearVelocity, &ControlledByPlayer), With<Structure>>,
    mut player_query: Query<(Entity, &mut LinearVelocity), (With<Player>, Without<Structure>)>,
    mut modules: Query<&mut LinearVelocity, (With<Module>, Without<Structure>, Without<Player>)>,
    player_resource: ResMut<PlayerResource>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
) {
    if player_resource.is_controlling_structure {
        let delta_time = time.delta_seconds();
        // Get structure controlled by player should be unique
        let (structure_entity, mut structure_velocity, controlled_by) = controlled_structure_query.single_mut();

        if let Ok((player_entity, mut player_velocity)) = player_query.get_mut(controlled_by.player_entity) {
            for event in input_reader.read() {
                match event {
                    InputAction::Move(direction) => {
                        structure_velocity.x += direction.x * 100.0 * delta_time;
                        structure_velocity.y += direction.y * 100.0 * delta_time;
                    }
                    _ => {}
                }
            }
            *player_velocity = structure_velocity.clone();

            for (mut module_velocity) in &mut modules {
                *module_velocity = structure_velocity.clone();
            }
        }
    }
}

fn debug_draw_structure_grid(mut gizmos: Gizmos, structures_query: Query<(&Transform, &Structure)>) {
    for (transform, structure) in &structures_query {
        let world_pos = transform.translation;
        let grid = &structure.grid;

        // Draw the grid
        gizmos
            .grid_2d(
                Vec2::new(world_pos.x, world_pos.y),
                0.0,
                UVec2::new(grid.width, grid.height),
                Vec2::splat(grid.cell_size),
                Color::from(GREY),
            )
            .outer_edges();
    }
}

fn debug_draw_player_inside_structure_rect(
    mut gizmos: Gizmos,
    query: Query<&GlobalTransform, With<Player>>,
    structures_query: Query<(&Transform, &Structure)>,
    player_resource: Res<PlayerResource>,
) {
    for player_transform in &query {
        for (structure_transform, structure) in &structures_query {
            let grid = &structure.grid;
            let square_size = grid.cell_size * 0.95; // Adjust this value to control the size of the square

            // Convert the adjusted position to grid coordinates
            let (grid_x, grid_y) = structure.world_to_grid(player_transform.translation(), structure_transform);

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
