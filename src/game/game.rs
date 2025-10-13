use crate::game::camera::PlayerCamera;
use crate::game::game_state::GameState;
use crate::game::levels::{LevelReadyEvent, PlayerSpawnPoint};
use crate::game::state::AppState;
use crate::type_expr;
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_player)
            .add_systems(OnExit(AppState::Game), remove_player)
            .add_systems(
                Update,
                (move_player, move_camera).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
pub struct Player;

fn add_player(
    _event: On<LevelReadyEvent>,
    mut cmd: Commands,
    spawn_point: Query<&Transform, With<PlayerSpawnPoint>>,
    asset_server: Res<AssetServer>,
) {
    let spawn_transform = match spawn_point.iter().next() {
        None => Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        Some(trans) => *trans,
    };

    cmd.spawn((
        Player,
        Mesh3d(asset_server.add(Sphere::new(0.125).mesh().build())),
        MeshMaterial3d(asset_server.add(type_expr!(
            StandardMaterial,
            Color::linear_rgb(0.0, 10.0, 12.0).into()
        ))),
        spawn_transform,
        RigidBody::Dynamic,
        Collider::sphere(0.125),
    ));
}

fn remove_player(mut cmd: Commands, players: Query<Entity, With<Player>>) {
    for player in players {
        cmd.entity(player).despawn();
    }
}

fn move_player(
    forces: Query<Forces, With<Player>>,
    camera: Single<&PlayerCamera>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut movement = Vec3::default();
    if key.pressed(KeyCode::KeyW) {
        movement += camera.get_looking();
    }
    if key.pressed(KeyCode::KeyS) {
        movement -= camera.get_looking();
    }
    if key.pressed(KeyCode::KeyA) {
        movement -= camera.get_looking().cross(Vec3::Y);
    }
    if key.pressed(KeyCode::KeyD) {
        movement += camera.get_looking().cross(Vec3::Y);
    }

    let torque = Vec3::Y.cross(movement.normalize_or_zero());

    for mut force in forces {
        force.apply_torque(torque);
    }
}

pub fn move_camera(
    mut camera: Single<(&mut Transform, &PlayerCamera)>,
    player: Single<&Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    let (ref mut transform, player_camera) = *camera;

    **transform = calculate_camera_transform(player.translation, player_camera);
}

fn calculate_camera_transform(player_pos: Vec3, player_camera: &PlayerCamera) -> Transform {
    let camera_offset = Vec3::new(
        player_camera.pitch.cos() * player_camera.yaw.sin(),
        -player_camera.pitch.sin(),
        player_camera.pitch.cos() * player_camera.yaw.cos(),
    ) * player_camera.distance;

    Transform::from_translation(player_pos + camera_offset).looking_at(player_pos, Vec3::Y)
}
