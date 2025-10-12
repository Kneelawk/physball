use crate::game::state::GameState;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode};
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Splash), setup_camera);
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Component, Reflect)]
pub struct PlayerCamera {
    pitch: f32,
    yaw: f32,
    distance: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        PlayerCamera {
            pitch: PI / 4.0,
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
        DespawnOnEnter(GameState::Splash),
    ));
}
