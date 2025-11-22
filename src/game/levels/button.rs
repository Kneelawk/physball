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
pub const BUTTON_DEBOUNCE: f32 = 0.1;

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
    Collider::cuboid(0.54, 0.05, 0.54),
    CollidingEntities::default()
)]
#[component(on_insert = level_button_plate_on_insert)]
pub struct LevelButtonPlate {
    pub default_trans: Transform,
    pub depressed_trans: Transform,
    pub depression: f32,
    pub debounce: f32,
    pub prev_debounce: f32,
    pub depressing: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Component)]
#[require(LevelObject, Transform)]
pub struct LevelButtonDoor {
    pub level: f32,
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
    button_plates: Query<(
        Entity,
        &CollidingEntities,
        &mut LevelButtonPlate,
        &mut Transform,
    )>,
    button_pressers: Query<(&Collider, &ColliderDensity), With<ButtonPresser>>,
    parents: Query<&ChildOf>,
    time: Res<Time>,
    preloads: Res<Preloads>,
) {
    for (plate_entity, colliding_entities, mut button_plate, mut plate_transform) in button_plates {
        let LevelButtonPlate {
            default_trans,
            depressed_trans,
            depression,
            debounce,
            prev_debounce,
            depressing,
        } = &mut *button_plate;

        let mut total_pressure = 0.0;
        for entity in colliding_entities.iter() {
            if let Ok((collider, density)) = button_pressers.get(*entity) {
                total_pressure += collider.mass(density.0);
            }
        }

        if total_pressure > 0.01 {
            *depression = (*depression + time.delta_secs() * BUTTON_DEPRESSION_SPEED).min(1.0);

            if !*depressing {
                *debounce = BUTTON_DEBOUNCE;
                *depressing = true;
            }
        } else {
            *depression = (*depression - time.delta_secs() * BUTTON_DEPRESSION_SPEED).max(0.0);

            if *depressing {
                *debounce = BUTTON_DEBOUNCE;
                *depressing = false;
            }
        }

        if *debounce < 0.0001
            && *prev_debounce > 0.0001
            && let Ok(parent) = parents.get(plate_entity)
        {
            let holder_entity = parent.parent();
            let mut commands = cmd.entity(holder_entity);
            commands.remove::<(AudioPlayer, AudioSink)>();

            if *depressing {
                commands.insert(AudioPlayer::new(preloads.button_on()));
            } else {
                commands.insert(AudioPlayer::new(preloads.button_off()));
            }
        }
        *prev_debounce = *debounce;

        *debounce = (*debounce - time.delta_secs()).max(0.0);

        *plate_transform = Transform::interpolate(&*default_trans, &*depressed_trans, *depression);
    }
}
