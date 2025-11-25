use crate::game::game_state::GameState;
use crate::game::input::PlayerInput;
use crate::game::levels::{LevelReadyEvent, LevelRestartEvent, PlayerSpawnPoint};
use crate::game::logic::{Player, spawn_transform};
use crate::game::state::AppState;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode};
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_start_level)
            .add_observer(on_restart_level)
            .add_systems(OnExit(AppState::Splash), setup_camera)
            .add_systems(
                Update,
                (rotate_camera, zoom_camera).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(AppState::Game), reset_camera);
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Component)]
pub struct PlayerCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        PlayerCamera {
            pitch: -PI / 8.0,
            yaw: 0.0,
            distance: 5.0,
        }
    }
}

impl PlayerCamera {
    pub fn get_looking(&self) -> Vec3 {
        -Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos())
    }
}

fn setup_camera(mut cmd: Commands) {
    cmd.spawn((
        PlayerCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Tonemapping::TonyMcMapface,
        Bloom {
            composite_mode: BloomCompositeMode::Additive,
            ..Bloom::NATURAL
        },
        DespawnOnEnter(AppState::Splash),
        SpatialListener::default(),
    ));
}

fn rotate_camera(
    mut camera: Single<&mut PlayerCamera>,
    mut mouse: MessageReader<PlayerInput>,
) {
    for mouse in mouse.read() {
        if let PlayerInput::CameraMovement(delta) = mouse {
            camera.yaw += -delta.x;
            camera.pitch = (camera.pitch - delta.y).clamp(-PI / 2.0 + 0.001, PI / 2.0 - 0.001);
        }
    }
}

fn zoom_camera(mut camera: Single<&mut PlayerCamera>, mut mouse: MessageReader<PlayerInput>) {
    for mouse in mouse.read() {
        if let PlayerInput::Zoom(delta) = mouse {
            camera.distance = (camera.distance - delta).clamp(0.05, 10.0);
        }
    }
}

fn on_start_level(
    _on: On<LevelReadyEvent>,
    mut camera: Single<&mut PlayerCamera>,
    spawn_point: Query<&Transform, (With<PlayerSpawnPoint>, Without<Player>)>,
) {
    **camera = apply_spawn_point_rotation(spawn_point);
}

fn on_restart_level(
    _on: On<LevelRestartEvent>,
    mut camera: Single<&mut PlayerCamera>,
    spawn_point: Query<&Transform, (With<PlayerSpawnPoint>, Without<Player>)>,
) {
    **camera = apply_spawn_point_rotation(spawn_point);
}

fn apply_spawn_point_rotation(
    spawn_point: Query<&Transform, (With<PlayerSpawnPoint>, Without<Player>)>,
) -> PlayerCamera {
    let spawn_transform = spawn_transform(spawn_point);
    let (axis, angle) = spawn_transform.rotation.to_axis_angle();
    let yaw = angle * axis.dot(Vec3::Y);
    PlayerCamera { yaw, ..default() }
}

fn reset_camera(mut camera: Single<&mut PlayerCamera>) {
    **camera = PlayerCamera::default();
}
