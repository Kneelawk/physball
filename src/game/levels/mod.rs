pub mod finish_point;

use crate::game::levels::finish_point::FinishPoint;
use crate::game::state::AppState;
use crate::type_expr;
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(respawn_level)
            .add_systems(OnExit(AppState::Game), (unselect_level, despawn_level))
            .add_systems(OnEnter(AppState::Game), spawn_level);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Event, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub struct LevelReadyEvent;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Event, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash)]
pub struct LevelRestartEvent;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Resource, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash, Resource)]
pub enum SelectedLevel {
    Level1,
    Level2,
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct LevelObject;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct PlayerSpawnPoint;

fn unselect_level(mut cmd: Commands) {
    cmd.remove_resource::<SelectedLevel>();
}

fn despawn_level(mut cmd: Commands, query: Query<Entity, With<LevelObject>>) {
    despawn_level_impl(&mut cmd, query);
}

fn despawn_level_impl(cmd: &mut Commands, query: Query<Entity, With<LevelObject>>) {
    for level in query {
        cmd.entity(level).despawn();
    }
}

fn respawn_level(
    _on: On<LevelRestartEvent>,
    mut cmd: Commands,
    query: Query<Entity, With<LevelObject>>,
    level: Res<SelectedLevel>,
    asset_server: Res<AssetServer>,
) {
    // TODO: Implement checkpoint system
    despawn_level_impl(&mut cmd, query);
    spawn_level_impl(&mut cmd, level, &asset_server);
}

fn spawn_level(mut cmd: Commands, level: Res<SelectedLevel>, asset_server: Res<AssetServer>) {
    spawn_level_impl(&mut cmd, level, &asset_server);

    cmd.trigger(LevelReadyEvent);
}

fn spawn_level_impl(cmd: &mut Commands, level: Res<SelectedLevel>, asset_server: &AssetServer) {
    match *level {
        SelectedLevel::Level1 => spawn_level1(cmd, asset_server),
        SelectedLevel::Level2 => spawn_level2(cmd, asset_server),
    }
}

fn spawn_level1(cmd: &mut Commands, asset_server: &AssetServer) {
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

    // cmd.spawn((
    //     DirectionalLight {
    //         illuminance: light_consts::lux::OVERCAST_DAY,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_rotation(
    //         Quat::from_rotation_x(-PI / 4.0) * Quat::from_rotation_z(-PI / 6.0),
    //     ),
    // ));
}

fn spawn_level2(cmd: &mut Commands, asset_server: &AssetServer) {
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

    cmd.spawn((FinishPoint, Transform::from_xyz(4.5, 0.5, 0.0)));

    // cmd.spawn((
    //     DirectionalLight {
    //         illuminance: light_consts::lux::OVERCAST_DAY,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_rotation(
    //         Quat::from_rotation_z(-PI / 6.0) * Quat::from_rotation_x(-PI / 4.0),
    //     ),
    // ));
}
