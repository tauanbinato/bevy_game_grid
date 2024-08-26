use crate::core::prelude::*;
use crate::world::prelude::*;

use crate::prelude::*;

const PROJECTILE_LIFETIME: f32 = 1.0;

pub struct StructuresCombatPlugin;

impl Plugin for StructuresCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_module_destroyed_system.run_if(on_event::<ModuleDestroyedEvent>()))
            .add_systems(
                Update,
                handle_depressurization_system
                    .run_if(on_event::<StructureDepressurizationEvent>())
                    .after(PhysicsSet::Sync),
            )
            .add_systems(FixedUpdate, structure_shoot_system.run_if(in_state(GameState::InGame)))
            .add_systems(
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
                density: 255.5,           // Density similar to steel or other high-density materials in kg/m^2
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
            ProjectileMaterialType::Ballistic => 0.5, // Desired diameter in meters (1 units in game, or 1 meter)
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

fn handle_depressurization_system(
    mut event_reader: EventReader<StructureDepressurizationEvent>,
    mut parent_query: Query<(&Children, &mut Pressurization, &mut Structure, &Transform)>,
    modules_query: Query<(Entity, &Module, &Transform)>,
    mut commands: Commands,
) {
    for event in event_reader.read() {
        // Ensure we are handling the correct structure
        if let Ok((children, mut pressurization, mut depressurized_structure, structure_transform)) =
            parent_query.get_mut(event.depressurized_structure)
        {
            let neighboring_modules =
                depressurized_structure.find_neighbors_of_exposed_modules(&pressurization.exposed_cells);

            for child in children.iter() {
                if let Ok((module_entity, module, module_transform)) = modules_query.get(*child) {
                    // Check if the module is in an exposed cell
                    if neighboring_modules.contains(&module.inner_grid_pos) {
                        // Calculate the direction of the force (from the structure's center to the module)
                        let direction_3d = (module_transform.translation - structure_transform.translation).normalize();
                        let direction = Vec2::new(direction_3d.x, direction_3d.y);

                        // Apply a simple force to simulate depressurization
                        let force_magnitude = 50000000.0; // Adjust this value as needed
                        let force = direction * force_magnitude;

                        commands.entity(module_entity).insert(ExternalForce::new(force).with_persistence(false));

                        commands.entity(module_entity).remove_parent_in_place();

                        // Handle depressurization: Make the module dynamic
                        commands.entity(module_entity).remove::<ColliderDensity>();
                        commands.entity(module_entity).insert(RigidBody::Dynamic);
                        commands.entity(module_entity).insert(Mass(20000.0));

                        // Set cell type to empty without this check_pressurization will not work properly
                        depressurized_structure
                            .grid
                            .set_cell_type_to_empty(module.inner_grid_pos.0, module.inner_grid_pos.1);
                    }
                }
            }
            let exposed_cells = depressurized_structure.check_pressurization();
            pressurization.exposed_cells = exposed_cells.clone();
        }
    }
}

fn handle_module_destroyed_system(
    parent: Query<&Parent>,
    mut parent_query: Query<(Entity, &mut Structure, &mut Pressurization)>,
    mut event_reader: EventReader<ModuleDestroyedEvent>,
    mut event_writer: EventWriter<StructureDepressurizationEvent>,
    mut commands: Commands,
) {
    // read teh event
    for event in event_reader.read() {
        // get the entity that was destroyed
        let module_destroyed = event.destroyed_entity;
        if let Ok(structure_parent) = parent.get(module_destroyed) {
            if let Ok((structure_entity, mut structure_attacked, mut pressurization)) =
                parent_query.get_mut(**structure_parent)
            {
                let module_inner_grid_pos = event.inner_grid_pos;
                // Remove from grid and check pressurization
                structure_attacked.grid.set_cell_type_to_empty(module_inner_grid_pos.0, module_inner_grid_pos.1);

                // Get the adjacent cells to the destroyed module
                let adjacent_cells = structure_attacked.get_adjacent_cells(module_inner_grid_pos);

                // Check if any adjacent cell is in the exposed_cells set from Pressurization
                let mut any_exposed = false;
                for adjacent_cell in adjacent_cells {
                    if !pressurization.exposed_cells.contains(&adjacent_cell) {
                        // if the module hit does not have near exposed cells, then could be a room pressurized or another module.
                        // we need to check if is a room or another module to call the event
                        if let Some(grid_cell) = structure_attacked.grid.get(adjacent_cell.0, adjacent_cell.1) {
                            if matches!(grid_cell.cell_type, CellType::Empty) {
                                // if the cell is empty, then is a room
                                any_exposed = true;
                                break;
                            }
                        }
                    }
                }
                let exposed_cells = structure_attacked.check_pressurization();
                pressurization.exposed_cells = exposed_cells.clone();

                if any_exposed {
                    event_writer.send(StructureDepressurizationEvent { depressurized_structure: structure_entity });
                }

                commands.entity(module_destroyed).remove_parent_in_place();
                despawn_entity(module_destroyed, &mut commands);
            }
        }
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
        //debug!("Projectile velocity: {:?}", projectile_vel.0.length());
        if timer.tick(time.delta()).just_finished() {
            despawn_entity(projectile_entity, &mut commands);
        }
    }
}

// TODO: Make a system to detect the collisions and emit an event of structure hit, this system will only listen to the event.
fn projectile_hit_system(
    mut collision_event_reader: EventReader<CollisionStarted>,
    projectile_physics_query: Query<(&LinearVelocity, &ProjectilePhysics), With<Projectile>>,
    mut module_physics_query: Query<&mut ModuleMaterial>,
    mut projectile_query: Query<&mut Projectile>,
    mut module_query: Query<&mut Module>,
    mut commands: Commands,
    mut event_writer: EventWriter<ModuleDestroyedEvent>,
) {
    for CollisionStarted(entity1, entity2) in collision_event_reader.read() {
        if let Some(projectile_entity) = find_matching_entity(*entity1, *entity2, &mut projectile_query) {
            if let Some(module_entity) = find_matching_entity(*entity1, *entity2, &mut module_query) {
                if let Some(module) = module_query.get(module_entity).ok() {
                    if let Ok((projectile_vel, projectile_physics)) = projectile_physics_query.get(projectile_entity) {
                        if let Ok(mut module_material) = module_physics_query.get_mut(module_entity) {
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
                                event_writer.send(ModuleDestroyedEvent {
                                    destroyed_entity: module_entity,
                                    inner_grid_pos: module.inner_grid_pos,
                                });
                            }

                            // // Debug output with all relevant information
                            // debug!(
                            //     "Collision Detected!\n\
                            //     Velocity: {:?} m/s\n\
                            //     Projectile Kinetic Energy: {:.2} J (joules)\n\
                            //     Module Material: {:?}\n\
                            //     Material Strength: {:.2} J\n\
                            //     Material Density: {:.2} kg/m²\n\
                            //     Projectile Material Density: {:.2} kg/m²\n\
                            //     Projectile Material Strength: {:.2} J\n\
                            //     Module Structural Points Before: {:.2}\n\
                            //     Damage Applied: {:.2}\n\
                            //     Module Structural Points After: {:.2} {}\n",
                            //     velocity_mps,
                            //     projectile_kinetic_energy,
                            //     module_material.material_type,
                            //     material_strength,
                            //     material_properties.density,
                            //     projectile_properties.density,
                            //     projectile_properties.yield_strength,
                            //     structural_points_before,
                            //     damage,
                            //     module_material.structural_points,
                            //     if is_destroyed { "(Destroyed)" } else { "" },
                            // );

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
                                let spawn_position = cannon_position + forward_direction * 3.0;

                                // Create the projectile physics object
                                let projectile_physics = ProjectilePhysics::ballistic(1.0);

                                let projectile_density = projectile_physics.density();

                                // Desired velocity in meters per second (m/s)
                                let desired_velocity_mps = 500.0;

                                // Calculate the impulse force using ProjectilePhysics
                                let impulse_force =
                                    projectile_physics.impulse_force(desired_velocity_mps, forward_direction);

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
