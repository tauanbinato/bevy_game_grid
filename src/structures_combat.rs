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

pub struct StructuresCombatPlugin;

impl Plugin for StructuresCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, structure_shoot_system.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
struct Projectile;

#[derive(Bundle)]
struct ProjectileBundle {
    projectile: Projectile,
    rigid_body: RigidBody,
    collider: Collider,
    mass: Mass,
    mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
    impulse: ExternalImpulse,
}

fn structure_shoot_system(
    mut query: Query<(&Transform, &Children), With<ControlledByPlayer>>,
    child_query: Query<(&Module, &Transform)>,
    mut input_reader: EventReader<InputAction>,
    time: Res<Time>,
    mut command: Commands,
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
                                // Calculate the forward direction based on the module's rotation
                                let forward_direction = structure_transform.rotation.mul_vec3(Vec3::Y).normalize();

                                // Combine the structure's global position with the module's local position
                                let spawn_position = structure_transform.translation
                                    + module_transform.translation
                                    + forward_direction * 35.0;

                                // Calculate the impulse force in the forward direction
                                let impulse_force = forward_direction * 100000.0;

                                command.spawn(ProjectileBundle {
                                    projectile: Projectile,
                                    rigid_body: RigidBody::Dynamic,
                                    collider: Collider::circle(5.0),
                                    mass: Mass(1.0),
                                    mesh_bundle: MaterialMesh2dBundle {
                                        material: materials.add(ColorMaterial::from(Color::from(WHITE))),
                                        mesh: meshes.add(Circle { radius: 5.0 }).into(),
                                        transform: Transform { translation: spawn_position, ..default() },
                                        visibility: Visibility::Inherited,
                                        ..default()
                                    },
                                    impulse: ExternalImpulse::new(impulse_force.truncate()).with_persistence(false),
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
