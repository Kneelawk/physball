//! World buttons that open doors.
//!
//! Note: this should be replaced by a level scene with an animation and a script at some point.

use crate::game::assets::preload::Preloads;
use crate::game::levels::LevelObject;
use avian3d::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;

#[derive(Debug, Default)]
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, detect_button_press);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
#[require(LevelObject, Transform, InheritedVisibility)]
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
    pub depression: f32,
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
#[require(
    LevelObject,
    Transform,
    RigidBody::Static,
    Sensor,
    Collider::cuboid(0.54, 0.06, 0.54),
    CollisionEventsEnabled
)]
pub struct LevelButtonSensor;

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
    commands.insert_if_new(SceneRoot(button_holder));
    commands.observe(level_button_on_scene_load);
    commands.with_children(|b| {
        b.spawn(LevelButtonPlate::default());
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
            && let Some(collider) = Collider::convex_decomposition_from_mesh(mesh)
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
    button_plates: Query<(Entity, &mut LevelButtonPlate, &mut Transform)>,
    contact_graph: Res<ContactGraph>,
) {
    for (plate_entity, mut button_plate, mut plate_transform) in button_plates {
        let mut total_pressure = 0.0;
        // FIXME: contacts don't generate impulses on non-dynamic bodies
        for contact_pair in contact_graph.contact_pairs_with(plate_entity) {
            total_pressure += contact_pair.total_normal_impulse_magnitude();
        }

        // info!("total_impulse: {total_pressure}");

        // if total_impulse > 1.0 {
        //     let depression = (1.0 - (total_impulse - 1.0) / 4.0).max(0.0);
        //     button_plate.depression = depression;
        //     plate_transform.translation = vec3(0.0, -0.04 * depression, 0.0);
        // }
    }
}
