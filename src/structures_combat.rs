use crate::inputs::InputAction;
use crate::modules::{MaterialProperties, Module, ModuleMaterial, ModuleMaterialType, ModuleType};
use crate::state::GameState;
use crate::structures::{ControlledByPlayer, Structure};
use crate::UNIT_SCALE;
use avian2d::math::Vector;
use avian2d::prelude::*;
use bevy::color::palettes::css::WHITE;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use std::any::Any;

const PROJECTILE_LIFETIME: f32 = 1.0;

pub struct StructuresCombatPlugin;

impl Plugin for StructuresCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, structure_shoot_system.run_if(in_state(GameState::InGame))).add_systems(
            Update,
            (projectile_hit_system, projectile_lifetime_system).chain().run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Debug, Default)]
enum ProjectileMaterialType {
    #[default]
    Ballistic,
    Explosive,
    Energy,
}

impl ProjectileMaterialType {
    fn properties(&self) -> MaterialProperties {
        match self {
            ProjectileMaterialType::Ballistic => MaterialProperties {
                strength: 300.0, // Strength in J/m³ for aluminum
                density: 28.0,   // Surface density in kg/m²
            },
            ProjectileMaterialType::Explosive => MaterialProperties {
                strength: 100.0, // Explosives might be less dense and more fragile
                density: 2000.0,
            },
            ProjectileMaterialType::Energy => MaterialProperties {
                strength: 500.0, // Energy projectiles could be more abstract, with high energy potential
                density: 1000.0, // Lower density
            },
        }
    }

    fn calculate_impulse_with_velocity(&self, velocity_mps: f32, mass: f32) -> f32 {
        // Convert the velocity to game units per second (GU/s)
        let velocity_gu = velocity_mps * UNIT_SCALE; // UNIT_SCALE = 10 (10 pixels = 1 meter)

        // Calculate the impulse (Impulse = Mass * Velocity)
        let impulse_gu_s = mass * velocity_gu;

        impulse_gu_s
    }

    fn size(&self) -> f32 {
        match self {
            ProjectileMaterialType::Ballistic => 1.0, // Desired diameter in meters (100 units in game, or 1 meter)
            ProjectileMaterialType::Energy => 0.5,
            ProjectileMaterialType::Explosive => 0.25,
        }
    }
}

#[derive(Debug, Default, Component)]
struct ProjectilePhysics {
    pub structural_points: f32,
    pub mass: f32,
    pub size: f32, // Diameter
    pub material_type: ProjectileMaterialType,
}

impl ProjectilePhysics {
    pub fn ballistic(scaling_factor: f32) -> Self {
        Self::create(ProjectileMaterialType::Ballistic, scaling_factor)
    }

    pub fn explosive(scaling_factor: f32) -> Self {
        Self::create(ProjectileMaterialType::Explosive, scaling_factor)
    }

    pub fn energy(scaling_factor: f32) -> Self {
        Self::create(ProjectileMaterialType::Energy, scaling_factor)
    }

    fn create(material_type: ProjectileMaterialType, scaling_factor: f32) -> Self {
        let diameter = material_type.size(); // Diameter in meters
        let radius = diameter / 2.0;

        // Calculate the area of the circle with the formula A = π * r²
        let area = std::f32::consts::PI * radius.powi(2);

        // Calculate the mass based on the material's density and the area
        let mass = material_type.properties().density * area;

        // Calculate structural points
        let structural_points = material_type.properties().strength * area * material_type.properties().density;

        Self {
            structural_points,
            mass,
            size: diameter * UNIT_SCALE, // Convert to game units
            material_type,
        }
    }

    pub fn debug_info(&self, impulse_force: Vec3) {
        let diameter = self.size;
        let radius = diameter / 2.0;
        let area = std::f32::consts::PI * (radius / UNIT_SCALE).powi(2);
        let density = self.mass / area;

        debug!(
            "Spawning Projectile: {:?}\n\
            Diameter: {:.2} meters\n\
            Radius: {:.2} meters\n\
            Mass: {:.2} kg\n\
            Area: {:.4} m²\n\
            Density: {:.2} kg/m²\n\
            Structural Points: {:.2}\n\
            Impulse Velocity: {:?} m/s
            ",
            self.material_type,
            diameter / UNIT_SCALE,
            radius / UNIT_SCALE,
            self.mass,
            area,
            density,
            self.structural_points,
            (impulse_force.length() / self.mass) / UNIT_SCALE
        );
    }
}

#[derive(Component, Deref, DerefMut)]
struct Projectile(Timer);

#[derive(Bundle)]
struct ProjectileBundle {
    projectile: Projectile,
    projectile_physics: ProjectilePhysics,
    rigid_body: RigidBody,
    collider: Collider,
    collider_density: ColliderDensity,
    mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
    impulse: ExternalImpulse,
    locked_axes: LockedAxes,
}

/// This function is used to find the entity that matches the query.
/// Given a query if the entity is found, it returns the entity, otherwise it returns `None`.
fn find_matching_entity<T: Component>(
    entity1: Entity,
    entity2: Entity,
    mut query: &mut Query<&mut T>,
) -> Option<Entity> {
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
            debug!("Projectile despawned due to lifetime expiration");
            despawn_entity(projectile_entity, &mut commands);
        }
    }
}

fn projectile_hit_system(
    mut collision_event_reader: EventReader<CollisionStarted>,
    projectile_physics_query: Query<(&LinearVelocity, &ProjectilePhysics), With<Projectile>>,
    mut module_physics_query: Query<(&mut ModuleMaterial, &Mass), With<Module>>,
    mut projectile_query: Query<&mut Projectile>,
    mut module_query: Query<&mut Module>,
    mut commands: Commands,
) {
    for CollisionStarted(entity1, entity2) in collision_event_reader.read() {
        if let Some(projectile_entity) = find_matching_entity(*entity1, *entity2, &mut projectile_query) {
            if let Some(module_entity) = find_matching_entity(*entity1, *entity2, &mut module_query) {
                if let Ok(mut module) = module_query.get_mut(module_entity) {
                    if let Ok((projectile_vel, projectile_physics)) = projectile_physics_query.get(projectile_entity) {
                        if let Ok((mut module_material, module_mass)) = module_physics_query.get_mut(module_entity) {
                            // Scale the velocity according to the game unit system
                            let scaled_velocity =
                                Vector::new(projectile_vel.0.x / UNIT_SCALE, projectile_vel.0.y / UNIT_SCALE);
                            // Calculate kinetic energy with the scaled velocity
                            let projectile_kinetic_energy =
                                (projectile_physics.mass * scaled_velocity.length_squared()) / 2.0;

                            // Retrieve the material's properties
                            let material_properties = module_material.material_type.properties();
                            let material_strength = material_properties.strength;

                            // Apply damage to the module's structural points
                            let damage = (projectile_kinetic_energy / material_strength);
                            module_material.structural_points -= damage;

                            // Check if the module is destroyed
                            if module_material.structural_points <= 0.0 {
                                despawn_entity(module_entity, &mut commands);
                            }

                            // Debug output with all relevant information
                            debug!(
                                "Collision Detected!\n\
                                Projectile Kinetic Energy: {:.2} J (joules)\n\
                                Module Material: {:?}\n\
                                Material Strength: {:.2} J\n\
                                Material Density: {:.2} kg/m^3\n\
                                Module Structural Points Before: {:.2}\n\
                                Damage Applied: {:.2}\n\
                                Module Structural Points After: {:.2}",
                                projectile_kinetic_energy,
                                module_material.material_type,
                                material_strength,
                                material_properties.density,
                                module_material.structural_points,
                                damage,
                                module_material.structural_points - damage
                            );

                            despawn_entity(projectile_entity, &mut commands);
                        }
                    }
                }
            }
        }
    }
}

fn structure_shoot_system(
    mut query: Query<(&Transform, &Children), With<ControlledByPlayer>>,
    child_query: Query<(&Module, &Transform)>,
    mut input_reader: EventReader<InputAction>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
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

                                let projectile_physics = ProjectilePhysics::ballistic(1.0);

                                let projectile_mass = projectile_physics.mass;
                                let projectile_size = projectile_physics.size;

                                // Desired velocity in meters per second (m/s)
                                let desired_velocity_mps = 1750.0; // Example: 1750 m/s for a tank round

                                // Calculate the impulse force in the forward direction using the helper function
                                let impulse_force = forward_direction
                                    * projectile_physics
                                        .material_type
                                        .calculate_impulse_with_velocity(desired_velocity_mps, projectile_mass);

                                projectile_physics.debug_info(impulse_force);

                                commands.spawn(ProjectileBundle {
                                    projectile: Projectile(Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once)),
                                    projectile_physics,
                                    rigid_body: RigidBody::Dynamic,
                                    collider: Collider::circle(projectile_size / 2.0),
                                    collider_density: ColliderDensity(
                                        projectile_mass / ((projectile_size / 2.0).powi(2) * std::f32::consts::PI),
                                    ),
                                    mesh_bundle: MaterialMesh2dBundle {
                                        material: materials.add(ColorMaterial::from(Color::from(WHITE))),
                                        mesh: meshes.add(Circle { radius: projectile_size / 2.0 }).into(),
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
