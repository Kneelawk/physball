use crate::game::state::AppState;
use crate::type_expr;
use avian3d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::Game), (unselect_level, despawn_level))
            .add_systems(OnEnter(AppState::Game), spawn_level);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Event, Reflect)]
pub struct LevelReadyEvent;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Resource, Reflect)]
pub enum SelectedLevel {
    Level1,
    Level2,
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
pub struct LevelObject;

fn unselect_level(mut cmd: Commands) {
    cmd.remove_resource::<SelectedLevel>();
}

fn despawn_level(mut cmd: Commands, query: Query<Entity, With<LevelObject>>) {
    for level in query {
        cmd.entity(level).despawn();
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
pub struct PlayerSpawnPoint;

fn spawn_level(cmd: Commands, level: Res<SelectedLevel>, asset_server: Res<AssetServer>) {
    match *level {
        SelectedLevel::Level1 => spawn_level1(cmd, &asset_server),
        SelectedLevel::Level2 => spawn_level2(cmd, &asset_server),
    }
}

fn spawn_level1(mut cmd: Commands, asset_server: &AssetServer) {
    cmd.spawn((
        LevelObject,
        Transform::default(),
        Mesh3d(asset_server.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)).into())),
        MeshMaterial3d(asset_server.add(type_expr!(StandardMaterial, Color::WHITE.into()))),
        children![(
            RigidBody::Static,
            Collider::cuboid(10.0, 0.2, 10.0),
            Transform::from_xyz(0.0, -0.1, 0.0)
        )],
    ));

    cmd.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0) * Quat::from_rotation_z(-PI / 6.0)),
    ));

    cmd.trigger(LevelReadyEvent);
}

fn spawn_level2(mut cmd: Commands, asset_server: &AssetServer) {
    cmd.spawn((
        LevelObject,
        Transform::default(),
        Mesh3d(asset_server.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)).into())),
        MeshMaterial3d(asset_server.add(type_expr!(StandardMaterial, Color::WHITE.into()))),
        children![(
            RigidBody::Static,
            Collider::cuboid(10.0, 0.2, 10.0),
            Transform::from_xyz(0.0, -0.1, 0.0)
        )],
    ));

    cmd.spawn((
        LevelObject,
        Transform::from_xyz(0.0, 0.25, -4.5),
        Mesh3d(asset_server.add(Cuboid::from_length(0.5).into())),
        MeshMaterial3d(asset_server.add(type_expr!(StandardMaterial, Color::WHITE.into()))),
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
    ));

    cmd.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_z(-PI / 6.0) * Quat::from_rotation_x(-PI / 4.0)),
    ));

    cmd.trigger(LevelReadyEvent);
}
