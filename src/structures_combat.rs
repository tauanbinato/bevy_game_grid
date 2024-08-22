use crate::inputs::InputAction;
use crate::modules::{Module, ModuleType};
use crate::state::GameState;
use crate::structures::{ControlledByPlayer, Structure};
use avian2d::math::Vector;
use avian2d::prelude::*;
use bevy::color::palettes::css::WHITE;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use std::any::Any;

pub struct StructuresCombatPlugin;

impl Plugin for StructuresCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, structure_shoot_system.run_if(in_state(GameState::InGame))).add_systems(
            Update,
            (projectile_hit_system, projectile_lifetime_system).chain().run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component, Deref, DerefMut)]
struct Projectile(Timer);

#[derive(Bundle)]
struct ProjectileBundle {
    projectile: Projectile,
    rigid_body: RigidBody,
    collider: Collider,
    mass: Mass,
    collider_density: ColliderDensity,
    mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
    impulse: ExternalImpulse,
    locked_axes: LockedAxes,
}

/// This function is used to find the entity that matches the query.
/// Given a query if the entity is found, it returns the entity, otherwise it returns `None`.
fn find_matching_entity<T: Component>(entity1: Entity, entity2: Entity, query: &Query<&T>) -> Option<Entity> {
    if query.get(entity1).is_ok() {
        Some(entity1)
    } else if query.get(entity2).is_ok() {
        Some(entity2)
    } else {
        None
    }
}

// Helper function to despawn an entity
fn despawn_entity(entity: Entity, commands: &mut Commands) {
    if commands.get_entity(entity).is_some() {
        commands.entity(entity).despawn();
    }
}

/// This system ticks the `Timer` on the entity with the `projectile_entity`
/// component using bevy's `Time` resource to get the delta between each update.
fn projectile_lifetime_system(time: Res<Time>, mut query: Query<(Entity, &mut Projectile)>, mut commands: Commands) {
    for (projectile_entity, mut timer) in &mut query {
        if timer.tick(time.delta()).just_finished() {
            despawn_entity(projectile_entity, &mut commands);
        }
    }
}

fn projectile_hit_system(
    mut collision_event_reader: EventReader<CollisionStarted>,
    projectile_query: Query<&Projectile>,
    module_query: Query<(&Module)>,
    mut commands: Commands,
) {
    for CollisionStarted(entity1, entity2) in collision_event_reader.read() {
        if let Some(projectile_entity) = find_matching_entity(*entity1, *entity2, &projectile_query) {
            if let Some(module_entity) = find_matching_entity(*entity1, *entity2, &module_query) {
                despawn_entity(projectile_entity, &mut commands);
                despawn_entity(module_entity, &mut commands);
            }
        }
    }
}

fn structure_shoot_system(
    mut query: Query<(&Transform, &Children), With<ControlledByPlayer>>,
    child_query: Query<(&Module, &Transform)>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let delta_time = time.delta_seconds();

    for event in input_reader.read() {
        match event {
            InputAction::Shoot => {
                for (structure_transform, childrens) in query.iter() {
                    for child in childrens {
                        if let Ok((module, module_transform)) = child_query.get(*child) {
                            if matches!(module.module_type, ModuleType::Cannon) {
                                // Determine the forward direction of the module in world space
                                let forward_direction = structure_transform
                                    .rotation
                                    .mul_vec3(module_transform.rotation.mul_vec3(Vec3::Y))
                                    .normalize();

                                // Calculate the global position of the cannon module
                                let cannon_position = structure_transform.translation
                                    + structure_transform.rotation.mul_vec3(module_transform.translation);

                                // Determine the spawn position a little in front of the cannon
                                let spawn_position = cannon_position + forward_direction * 35.0;

                                // Calculate the impulse force in the forward direction
                                let impulse_force = forward_direction * 300000.0;

                                commands.spawn(ProjectileBundle {
                                    projectile: Projectile(Timer::from_seconds(2.0, TimerMode::Once)),
                                    rigid_body: RigidBody::Dynamic,
                                    collider: Collider::circle(5.0),
                                    collider_density: ColliderDensity(0.0),
                                    mass: Mass(100.0),
                                    mesh_bundle: MaterialMesh2dBundle {
                                        material: materials.add(ColorMaterial::from(Color::from(WHITE))),
                                        mesh: meshes.add(Circle { radius: 5.0 }).into(),
                                        transform: Transform { translation: spawn_position, ..default() },
                                        visibility: Visibility::Inherited,
                                        ..default()
                                    },
                                    impulse: ExternalImpulse::new(impulse_force.truncate()).with_persistence(false),
                                    locked_axes: LockedAxes::ROTATION_LOCKED,
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
