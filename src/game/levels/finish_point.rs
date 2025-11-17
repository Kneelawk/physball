use crate::game::assets::fonts::FontNames;
use crate::game::assets::preload::Preloads;
use crate::game::logic::Player;
use crate::game::game_state::GameState;
use crate::game::levels::LevelObject;
use avian3d::prelude::*;
use bevy::asset::io::embedded::GetAssetServer;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAtlas};

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FinishPointPlugin;

impl Plugin for FinishPointPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(detect_level_finish)
            .add_systems(Update, spin_finish_sign);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[require(
    LevelObject,
    Transform,
    Sensor,
    Collider::cuboid(1.0, 1.0, 1.0),
    CollisionEventsEnabled
)]
#[component(on_insert = finish_point_on_insert)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct FinishPoint;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct FinishLabel;

fn finish_point_on_insert(mut world: DeferredWorld, ctx: HookContext) {
    let preloads = world
        .get_resource::<Preloads>()
        .expect("missing preloads resource");
    let font_names = world
        .get_resource::<FontNames>()
        .expect("missing font names resource");
    let level_end = preloads.level_end();
    let font = preloads.title_font();
    let font = font_names
        .get(&font.id())
        .expect("missing font name for required font")
        .clone();
    let material = world.get_asset_server().add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        emissive: LinearRgba::new(0.0, 10.0, 12.0, 1.0),
        ..default()
    });
    let mut binding = world.commands();
    let mut commands = binding.entity(ctx.entity);
    commands.insert_if_new(SceneRoot(level_end));
    commands.with_child((
        FinishLabel,
        Transform::from_xyz(0.0, 0.65, 0.0),
        Text3d::new("Finish"),
        Text3dStyling {
            font: font.into(),
            size: 64.0,
            world_scale: Some(Vec2::splat(0.15)),
            ..default()
        },
        Mesh3d::default(),
        MeshMaterial3d(material),
    ));
}

fn detect_level_finish(
    collision: On<CollisionStart>,
    query: Query<(), With<FinishPoint>>,
    player_query: Query<(), With<Player>>,
    mut state: ResMut<NextState<GameState>>,
) {
    if query.contains(collision.collider1) && player_query.contains(collision.collider2) {
        info!("Level finished");
        state.set(GameState::Finished);
    }
}

fn spin_finish_sign(query: Query<&mut Transform, With<FinishLabel>>, time: Res<Time>) {
    for mut trans in query {
        trans.rotation = Quat::from_rotation_y(time.elapsed_secs_wrapped());
    }
}
