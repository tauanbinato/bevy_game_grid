use crate::core::state::GameState;
use crate::world::prelude::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerResource::default())
            .add_systems(OnEnter(GameState::BuildingStructures), spawn_camera)
            .add_systems(
                PostUpdate,
                (update_player_camera, update_structure_camera)
                    .run_if(in_state(GameState::InGame))
                    .after(PhysicsSet::Sync)
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

/// Camera lerp factor.
const CAM_LERP_FACTOR: f32 = 2.0;
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0)),
        //projection: OrthographicProjection { scaling_mode: ScalingMode::WindowSize(10.0), ..default() },
        ..Default::default()
    });
}

/// Update the camera position by tracking the player.
fn update_player_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&GlobalTransform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
    player_resource: Res<PlayerResource>,
) {
    if player_resource.is_controlling_structure {
        return;
    }

    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let Vec3 { x, y, .. } = player.translation();
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using interpolation between
    // the camera position and the player position on the x and y axes.
    // Here we use the in-game time, to get the elapsed time (in seconds)
    // since the previous update. This avoids jittery movement when tracking
    // the player.
    camera.translation = camera.translation.lerp(direction, time.delta_seconds() * CAM_LERP_FACTOR);
}

fn update_structure_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<ControlledByPlayer>)>,
    structure: Query<(&GlobalTransform, &LinearVelocity), (With<ControlledByPlayer>, Without<Camera2d>)>,
    time: Res<Time>,
    player_resource: Res<PlayerResource>,
) {
    if !player_resource.is_controlling_structure {
        return;
    }

    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    for (structure, linear_vel) in structure.iter() {
        let Vec3 { x, y, .. } = structure.translation();
        let direction = Vec3::new(x, y, camera.translation.z);

        camera.translation = direction;
    }
}
