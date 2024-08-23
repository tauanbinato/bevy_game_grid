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
                yield_strength: 250000.0, // Strength in J/m³ for ballistic materials (higher due to the kinetic nature)
                density: 78.5,            // Density similar to steel or other high-density materials in kg/m^2
                thickness: 0.01,          // Typical thickness for ballistic projectiles
                damage_threshold: 30000.0, // Damage threshold for ballistic impacts
            },
            ProjectileMaterialType::Explosive => MaterialProperties {
                yield_strength: 50.0,      // Explosives are less dense and more fragile
                density: 10.0,             // Density in kg/m^3, varies depending on the type of explosive
                thickness: 0.02,           // Thickness could represent the effective range or blast radius
                damage_threshold: 50000.0, // Higher threshold due to the explosive nature
            },
            ProjectileMaterialType::Energy => MaterialProperties {
                yield_strength: 0.0,        // Energy projectiles have no physical yield strength
                density: 0.0,               // Density is irrelevant for pure energy projectiles
                thickness: 0.0,             // Thickness is not applicable
                damage_threshold: 100000.0, // Extremely high damage potential
            },
        }
    }

    fn size(&self) -> f32 {
        match self {
            ProjectileMaterialType::Ballistic => 1.0, // Desired diameter in meters (10 units in game, or 1 meter)
            ProjectileMaterialType::Energy => 0.5,
            ProjectileMaterialType::Explosive => 0.25,
        }
    }
}

#[derive(Debug, Default, Component)]
struct ProjectilePhysics {
    pub structural_points: f32,
    pub mass: f32,
    pub size: f32, // Diameter in meters
    pub area: f32, // Area in square meters
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
        // Diameter in game units (pixels)
        let diameter = material_type.size() * UNIT_SCALE; // Convert diameter to game units immediately
        let radius = diameter / 2.0;

        // Calculate the area of the circle in game units (pixels²)
        let area = std::f32::consts::PI * radius.powi(2);

        // Calculate the mass based on the material's density and the area (mass in game units)
        let mass = material_type.properties().density * area;

        // Calculate structural points (using game units for area)
        let structural_points = material_type.properties().yield_strength * area * material_type.properties().density;

        // Debug output to verify calculations
        debug!(
            "Projectile Created - Type: {:?}, Diameter: {:.2}px, Area: {:.4}px², Mass: {:.2}, Game Size: {:.2}px",
            material_type, diameter, area, mass, diameter
        );

        Self {
            area,              // Area in game units (pixels²)
            structural_points, // Structural points based on game units
            mass,              // Mass in game units
            size: diameter,    // Size in game units (pixels)
            material_type,
        }
    }

    pub fn density(&self) -> f32 {
        // Calculate the area using the size in game units (pixels)
        let game_area = std::f32::consts::PI * (self.size / 2.0).powi(2);

        // Calculate the density using mass and the area in game units
        self.mass / game_area
    }

    pub fn impulse_force(&self, desired_velocity_mps: f32, forward_direction: Vec3) -> Vec3 {
        // Calculate the impulse force needed to achieve the desired velocity
        forward_direction * (self.mass * desired_velocity_mps) * UNIT_SCALE
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
            impulse_force.length()
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
fn projectile_lifetime_system(
    time: Res<Time>,
    mut query: Query<(Entity, &LinearVelocity, &mut Projectile)>,
    mut commands: Commands,
) {
    for (projectile_entity, projectile_vel, mut timer) in &mut query {
        debug!("Projectile velocity: {:?}", projectile_vel.0.length());
        if timer.tick(time.delta()).just_finished() {
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
                            // No need to scale the velocity; it's already in m/s.
                            let velocity_mps = (projectile_vel.0.length());

                            // Calculate the kinetic energy of the projectile (Joules)
                            let projectile_kinetic_energy = 0.5 * projectile_physics.mass * velocity_mps.powi(2);

                            // Retrieve the material's properties for the module and projectile
                            let material_properties = module_material.material_type.properties();
                            let projectile_properties = projectile_physics.material_type.properties();

                            let material_strength = material_properties.yield_strength;

                            // Factor in the projectile's density and yield strength
                            let density_factor = projectile_properties.density / material_properties.density;
                            let hardness_factor =
                                projectile_properties.yield_strength / material_properties.yield_strength;

                            // Calculate the adjusted damage
                            let damage =
                                (projectile_kinetic_energy * density_factor * hardness_factor) / material_strength;

                            // Update the module's structural points
                            let structural_points_before = module_material.structural_points;
                            module_material.structural_points -= damage;

                            // Check if the module is destroyed
                            let is_destroyed = module_material.structural_points <= 0.0;
                            if is_destroyed {
                                despawn_entity(module_entity, &mut commands);
                            }

                            // Debug output with all relevant information
                            debug!(
                                "Collision Detected!\n\
                            Velocity: {:?} m/s\n\
                            Projectile Kinetic Energy: {:.2} J (joules)\n\
                            Module Material: {:?}\n\
                            Material Strength: {:.2} J\n\
                            Material Density: {:.2} kg/m²\n\
                            Projectile Material Density: {:.2} kg/m²\n\
                            Projectile Material Strength: {:.2} J\n\
                            Module Structural Points Before: {:.2}\n\
                            Damage Applied: {:.2}\n\
                            Module Structural Points After: {:.2} {}\n",
                                velocity_mps,
                                projectile_kinetic_energy,
                                module_material.material_type,
                                material_strength,
                                material_properties.density,
                                projectile_properties.density,
                                projectile_properties.yield_strength,
                                structural_points_before,
                                damage,
                                module_material.structural_points,
                                if is_destroyed { "(Destroyed)" } else { "" },
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

                                // Create the projectile physics object
                                let projectile_physics = ProjectilePhysics::ballistic(1.0);

                                let projectile_density = projectile_physics.density();

                                // Desired velocity in meters per second (m/s)
                                let desired_velocity_mps = 1750.0;

                                // Calculate the impulse force using ProjectilePhysics
                                let impulse_force =
                                    projectile_physics.impulse_force(desired_velocity_mps, forward_direction);

                                // Debug output to verify impulse force
                                debug!(
                                    "Impulse Force: {:.2} N·s, Desired Velocity: {:.2} m/s",
                                    impulse_force.length(),
                                    desired_velocity_mps
                                );

                                let projectile_size = projectile_physics.size;

                                commands.spawn(ProjectileBundle {
                                    projectile: Projectile(Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once)),
                                    projectile_physics,
                                    rigid_body: RigidBody::Dynamic,
                                    collider: Collider::circle(projectile_size / 2.0),
                                    collider_density: ColliderDensity(projectile_density),
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
