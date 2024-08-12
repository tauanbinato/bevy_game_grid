use std::process::Command;
use avian2d::{prelude::*};
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
        app.add_event::<ModuleInteractionEvent>()
            .add_systems(OnEnter(GameState::BuildingStructures), setup_structures_from_file)
            .add_systems(Update, (control_command_center_system, move_structure_system).run_if(in_state(GameState::InGame)))
            .add_systems(FixedUpdate, move_structure_system.run_if(in_state(GameState::InGame)));

        if self.debug_enable {
            app.add_systems(Update, (debug_draw_structure_grid,debug_draw_player_rect_grid_in_structure).chain().run_if(in_state(GameState::InGame)));
        }
    }
}

#[derive(Component)]
struct ControlledByPlayer {
    player_entity: Entity,
}

#[derive(Debug, Default)]
struct Module {
    inner_grid_pos: (i32, i32),
    module_type: ModuleType,
    entity_controlling: Option<Entity>,
}

#[derive(Debug, Default)]
enum ModuleType {
    #[default]
    Walkable,
    Engine,
    CommandCenter,
    LivingQuarters,
    Storage,
    Wall,
}

#[derive(Bundle)]
struct StructureBundle {
    rigid_body: RigidBody,
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
                                ..default()
                            };
                            structure_component.add_module(engine_module);
                        },
                        'C' => {
                            let command_center_module = Module {
                                inner_grid_pos: (x as i32, y as i32),
                                module_type: ModuleType::CommandCenter,
                                ..default()
                            };
                            structure_component.add_module(command_center_module);
                        },
                        'W' => {
                            let walkable_module = Module {
                                inner_grid_pos: (x as i32, y as i32),
                                module_type: ModuleType::Walkable,
                                ..default()
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
                rigid_body: RigidBody::Dynamic,
                structure: structure_component,
                transform: Transform::from_translation(Vec3::new(-500.0, 100.0, 1.0)),
            });
        }
        next_state.set(GameState::InGame);
    } else {
        panic!("Failed to load structures asset");
    }
}

#[derive(Event)]
pub enum ModuleInteractionEvent {
    TakeControl {
        player_entity: Entity,
        structure_entity: Entity,
    },
    ReleaseControl {
        player_entity: Entity,
        structure_entity: Entity,
    },
}

fn control_command_center_system(
    mut event_reader: EventReader<InputAction>,
    mut event_writer: EventWriter<ModuleInteractionEvent>,
    mut structure_query: Query<(Entity, &mut Structure, &Transform)>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut command: Commands
) {

    //loop for player pos
    for (player_entity, player_transform) in &player_query {
        for (structure_entity, mut structure, structure_transform) in &mut structure_query {
            let player_world_pos = player_transform.translation - structure_transform.translation;
            let player_grid_pos = structure.grid.world_to_grid(player_world_pos);

            // Check if the player is within the grid boundaries
            if player_grid_pos.0 >= 0 && player_grid_pos.0 < structure.grid.width as i32 &&
                player_grid_pos.1 >= 0 && player_grid_pos.1 < structure.grid.height as i32 {

                // Check if the player is in a Command Center and if so, check if the player is already controlling it
                if let Some(command_center_module) = structure.modules.iter_mut().find(|module| {

                    matches!(module.module_type, ModuleType::CommandCenter) &&
                        matches!(module.module_type, ModuleType::CommandCenter) && // Checking if the module is a Command Center
                        module.inner_grid_pos == player_grid_pos // Checking if the player is in the Command Center
                }) {
                    // Player can control or release the Command Center by pressing the spacebar.
                    for event in event_reader.read() {
                        if let InputAction::SpacePressed = event {
                            if command_center_module.entity_controlling.is_none() {
                                // Take control if no one is controlling it
                                command_center_module.entity_controlling = Some(player_entity);
                                debug!("Player is now controlling the Command Center.");


                                // lets insert the PlayerControlled component to the structure
                                command.entity(structure_entity).insert(ControlledByPlayer {
                                    player_entity,
                                });

                                event_writer.send(ModuleInteractionEvent::TakeControl {
                                    player_entity,
                                    structure_entity,
                                });
                            } else if command_center_module.entity_controlling == Some(player_entity) {
                                // Release control if the player is already controlling it
                                command_center_module.entity_controlling = None;
                                debug!("Player has released control of the Command Center.");

                                // lets remove the PlayerControlled component from the structure
                                command.entity(structure_entity).remove::<ControlledByPlayer>();

                                // Emit an event for releasing control
                                event_writer.send(ModuleInteractionEvent::ReleaseControl {
                                    player_entity,
                                    structure_entity,
                                });
                            }
                        }
                    }
                } else {
                    // debug!("Player is not in a Command Center or the is not the module.");
                }
            }
        }
    }

}

fn move_structure_system(
    mut controlled_structures_query: Query<(Entity, &mut LinearVelocity, &ControlledByPlayer), With<Structure>>,
    player_query: Query<(Entity, &LinearVelocity), (With<Player>, Without<Structure>)>,
) {

    // Loop through all structures that are controlled by the player
    for (structure_entity, mut structure_velocity, controlled_by) in &mut controlled_structures_query {
        if let Ok((player_entity, player_velocity)) = player_query.get(controlled_by.player_entity) {
            // Set the structure's velocity to match the player's velocity
            *structure_velocity = *player_velocity;
        }
    }
}

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
                ModuleType::Wall => BLACK,
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