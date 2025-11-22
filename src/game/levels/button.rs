//! World buttons that open doors.
//!
//! Note: this should be replaced by a level scene with an animation and a script at some point.

use crate::game::assets::preload::Preloads;
use crate::game::levels::LevelObject;
use crate::game::state::AppState;
use avian3d::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;

// needs to be less than gravity so we don't have oscillations
pub const BUTTON_DEPRESSION_SPEED: f32 = 8.0;
pub const DOOR_SLIDE_SPEED: f32 = 1.25;

const PLAYBACK_SETTINGS: PlaybackSettings = PlaybackSettings::REMOVE.with_spatial(true);

#[derive(Debug, Default)]
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, detect_button_press.run_if(in_state(AppState::Game)));
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct ButtonPresser;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
#[component(storage = "SparseSet")]
pub struct PressedButton;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
#[require(
    LevelObject,
    Transform,
    InheritedVisibility,
    PlaybackSettings = PlaybackSettings::REMOVE.with_spatial(true)
)]
#[component(on_insert = level_button_on_insert)]
pub struct LevelButton;

#[derive(Debug, Default, Clone, PartialEq, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Component)]
#[require(
    LevelObject,
    Transform,
    RigidBody::Kinematic,
    Collider::cuboid(0.54, 0.05, 0.54)
)]
#[component(on_insert = level_button_plate_on_insert)]
pub struct LevelButtonPlate {
    pub default_trans: Transform,
    pub depressed_trans: Transform,
    pub depression: f32,
}

#[derive(Debug, Default, Clone, PartialEq, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Component)]
#[require(
    LevelObject,
    Transform,
    RigidBody::Static,
    Collider::cuboid(0.54, 0.08, 0.54),
    Sensor,
    CollidingEntities::default()
)]
pub struct LevelButtonSensor {
    pub prev_sensor_pressed: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Component)]
#[require(LevelObject, Transform)]
pub struct LevelButtonDoor {
    pub default_trans: Transform,
    pub open_trans: Transform,
    pub openness: f32,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash, Component)]
#[relationship(relationship_target = ControlledObjects)]
pub struct ControlledBy(pub Entity);

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash, Component)]
#[relationship_target(relationship = ControlledBy)]
pub struct ControlledObjects(Vec<Entity>);

impl ControlledObjects {
    pub fn controlled(&self) -> &[Entity] {
        &self.0
    }
}

fn level_button_on_insert(mut world: DeferredWorld, ctx: HookContext) {
    let preloads = world
        .get_resource::<Preloads>()
        .expect("missing preloads resource");
    let button_holder = preloads.button_holder();
    let mut binding = world.commands();
    let mut commands = binding.entity(ctx.entity);
    commands.with_children(|b| {
        b.spawn(SceneRoot(button_holder))
            .observe(level_button_on_scene_load);
        b.spawn((
            LevelButtonPlate {
                default_trans: Transform::from_xyz(0.0, 0.035, 0.0),
                depressed_trans: Transform::from_xyz(0.0, -0.005, 0.0),
                ..default()
            },
            Transform::from_xyz(0.0, 0.035, 0.0),
        ));
        b.spawn((
            LevelButtonSensor::default(),
            Transform::from_xyz(0.0, 0.035, 0.0),
        ));
    });
}

fn level_button_on_scene_load(
    scene: On<SceneInstanceReady>,
    mut cmd: Commands,
    children: Query<&Children>,
    mesh_handles: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
) {
    for child in children.iter_descendants(scene.entity) {
        if let Ok(mesh3d) = mesh_handles.get(child)
            && let Some(mesh) = meshes.get(&mesh3d.0)
            && let Some(collider) = Collider::trimesh_from_mesh(mesh)
        {
            cmd.entity(child).insert((RigidBody::Static, collider));
        }
    }
}

fn level_button_plate_on_insert(mut world: DeferredWorld, ctx: HookContext) {
    let preloads = world
        .get_resource::<Preloads>()
        .expect("missing preloads resource");
    let button_plate = preloads.button_plate();
    let mut binding = world.commands();
    let mut commands = binding.entity(ctx.entity);
    commands.insert_if_new(SceneRoot(button_plate));
}

fn detect_button_press(
    mut cmd: Commands,
    buttons: Query<Entity, With<LevelButton>>,
    children: Query<&Children>,
    controlled: Query<&ControlledObjects>,
    mut button_plates: Query<(&mut LevelButtonPlate, &mut Transform), Without<LevelButtonDoor>>,
    mut button_sensors: Query<(&CollidingEntities, &mut LevelButtonSensor)>,
    button_pressers: Query<(), With<ButtonPresser>>,
    mut button_doors: Query<(&mut LevelButtonDoor, &mut Transform), Without<LevelButtonPlate>>,
    time: Res<Time>,
    preloads: Res<Preloads>,
) {
    for button in buttons {
        let Some(plate_entity) = children
            .get(button)
            .iter()
            .flat_map(|children| children.iter())
            .find(|child| button_plates.contains(*child))
        else {
            continue;
        };
        let (mut button_plate, mut plate_transform) = button_plates
            .get_mut(plate_entity)
            .expect("button_plates does not have a contained entity");
        let Some(sensor_entity) = children
            .get(button)
            .iter()
            .flat_map(|children| children.iter())
            .find(|child| button_sensors.contains(*child))
        else {
            continue;
        };
        let (sensor_collisions, mut button_sensor) = button_sensors
            .get_mut(sensor_entity)
            .expect("button_sensors does not have a contained entity");
        let LevelButtonPlate {
            default_trans,
            depressed_trans,
            depression,
        } = &mut *button_plate;

        let mut sensor_pressed = false;
        for entity in sensor_collisions.iter() {
            if button_pressers.contains(*entity) {
                sensor_pressed = true;
                break;
            }
        }

        if sensor_pressed {
            *depression = (*depression + time.delta_secs() * BUTTON_DEPRESSION_SPEED).min(1.0);
        } else {
            *depression = (*depression - time.delta_secs() * BUTTON_DEPRESSION_SPEED).max(0.0);
        }

        *plate_transform = Transform::interpolate(&*default_trans, &*depressed_trans, *depression);

        if button_sensor.prev_sensor_pressed != sensor_pressed {
            let mut commands = cmd.entity(button);
            commands.remove::<(AudioPlayer, SpatialAudioSink, PlaybackSettings)>();

            if sensor_pressed {
                commands.insert((AudioPlayer::new(preloads.button_on()), PLAYBACK_SETTINGS));
            } else {
                commands.insert((AudioPlayer::new(preloads.button_off()), PLAYBACK_SETTINGS));
            }
        }

        for door_entity in controlled
            .get(button)
            .iter()
            .flat_map(|controlled| controlled.controlled().iter())
            .copied()
        {
            if let Ok((mut door, mut door_trans)) = button_doors.get_mut(door_entity) {
                if sensor_pressed {
                    door.openness = (door.openness + time.delta_secs() * DOOR_SLIDE_SPEED).min(1.0);
                } else {
                    door.openness = (door.openness - time.delta_secs() * DOOR_SLIDE_SPEED).max(0.0);
                }

                *door_trans =
                    Transform::interpolate(&door.default_trans, &door.open_trans, door.openness);

                if button_sensor.prev_sensor_pressed != sensor_pressed {
                    let mut commands = cmd.entity(door_entity);
                    commands.remove::<(AudioPlayer, SpatialAudioSink, PlaybackSettings)>();

                    if sensor_pressed {
                        commands
                            .insert((AudioPlayer::new(preloads.door_open()), PLAYBACK_SETTINGS));
                    } else {
                        commands
                            .insert((AudioPlayer::new(preloads.door_close()), PLAYBACK_SETTINGS));
                    }
                }
            }
        }

        button_sensor.prev_sensor_pressed = sensor_pressed;
    }
}
