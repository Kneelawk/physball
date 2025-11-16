use crate::game::assets::preload::Preloads;
use crate::game::camera::PlayerCamera;
use crate::game::game_state::GameState;
use crate::game::levels::death::{Kill, Killable, PlayerDiedEvent};
use crate::game::levels::{LevelReadyEvent, LevelRestartEvent, PlayerSpawnPoint};
use crate::game::state::AppState;
use avian3d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

pub const MOVEMENT_ACCELERATION: f32 = 30.0 * PI;
pub const JUMP_VELOCITY: f32 = 4.0;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_player)
            .add_observer(reset_player)
            .add_observer(on_collision_start)
            .add_observer(on_collision_stop)
            .add_systems(OnExit(AppState::Game), remove_player)
            .add_systems(
                Update,
                (move_player, jump_player, kill_player).run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, move_camera.run_if(in_state(AppState::Game)));
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct Player;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

pub fn spawn_transform(
    spawn_point: Query<&Transform, (With<PlayerSpawnPoint>, Without<Player>)>,
) -> Transform {
    let spawn_point = spawn_point.iter().copied().collect::<Vec<_>>();
    match spawn_point.iter().next() {
        None => Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        Some(trans) => *trans,
    }
}

fn add_player(
    _event: On<LevelReadyEvent>,
    mut cmd: Commands,
    spawn_point: Query<&Transform, (With<PlayerSpawnPoint>, Without<Player>)>,
    preloads: Res<Preloads>,
) {
    let spawn_transform = spawn_transform(spawn_point);

    let collider = Collider::sphere(0.25);

    cmd.spawn((
        Player,
        spawn_transform,
        RigidBody::Dynamic,
        collider,
        AngularDamping(0.25),
        LinearDamping(0.25),
        CollisionEventsEnabled,
        Killable,
        children![SceneRoot(preloads.physball()),],
    ));
}

fn reset_player(
    _on: On<LevelRestartEvent>,
    spawn_point: Query<&Transform, (With<PlayerSpawnPoint>, Without<Player>)>,
    player: Query<
        (&mut Transform, &mut AngularVelocity, &mut LinearVelocity),
        (With<Player>, Without<PlayerSpawnPoint>),
    >,
) {
    let spawn_transform = spawn_transform(spawn_point);

    for (mut transform, mut ang_vel, mut lin_vel) in player {
        *transform = spawn_transform;
        ang_vel.0 = Vec3::ZERO;
        lin_vel.0 = Vec3::ZERO;
    }
}

fn remove_player(mut cmd: Commands, players: Query<Entity, With<Player>>) {
    for player in players {
        cmd.entity(player).despawn();
    }
}

fn move_player(
    forces: Query<&mut AngularVelocity, With<Player>>,
    camera: Single<&PlayerCamera>,
    key: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
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

    if torque.length_squared() > 0.001 {
        for mut force in forces {
            force.0 += torque * time.delta_secs() * MOVEMENT_ACCELERATION;
        }
    }
}

fn on_collision_start(
    on: On<CollisionStart>,
    filter: Query<(), With<Player>>,
    cmd: Commands,
    collisions: Collisions,
) {
    if filter.contains(on.collider1) {
        update_grounded(cmd, on.collider1, collisions);
    }
}

fn on_collision_stop(
    on: On<CollisionEnd>,
    filter: Query<(), With<Player>>,
    cmd: Commands,
    collisions: Collisions,
) {
    if filter.contains(on.collider1) {
        update_grounded(cmd, on.collider1, collisions);
    }
}

// Copied from Avian3d example
fn update_grounded(mut cmd: Commands, player: Entity, collisions: Collisions) {
    let is_grounded = collisions
        .collisions_with(player)
        .filter(|pair| pair.is_touching() && pair.generates_constraints())
        .flat_map(|pair| {
            pair.manifolds.iter().map(|manifold| {
                if pair.collider1 == player {
                    -manifold.normal
                } else {
                    manifold.normal
                }
            })
        })
        .any(|hit| hit.angle_between(Vec3::Y).abs() <= PI * 2.0 / 3.0);

    if is_grounded {
        cmd.entity(player).insert(Grounded);
    } else {
        cmd.entity(player).remove::<Grounded>();
    }
}

// Copied from Avian3d example
fn jump_player(
    forces: Query<(&mut LinearVelocity, Has<Grounded>), With<Player>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Space) {
        for (mut vel, grounded) in forces {
            if grounded {
                vel.y = JUMP_VELOCITY;
            }
        }
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

fn kill_player(
    mut kill_msg: MessageReader<Kill>,
    player: Query<(), With<Player>>,
    mut cmd: Commands,
) {
    for kill in kill_msg.read() {
        if player.contains(kill.to_kill) {
            info!("Player died.");
            cmd.trigger(PlayerDiedEvent);
            cmd.trigger(LevelRestartEvent);

            kill_msg.clear();
            return;
        }
    }
}
