use crate::game::assets::preload::Preloads;
use crate::game::game::Player;
use crate::game::game_state::GameState;
use crate::game::levels::LevelObject;
use avian3d::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FinishPointPlugin;

impl Plugin for FinishPointPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(detect_level_finish);
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

fn finish_point_on_insert(mut world: DeferredWorld, ctx: HookContext) {
    let level_end = world
        .get_resource::<Preloads>()
        .unwrap()
        .get("level-end")
        .expect("missing required preload");
    let level_end = level_end
        .clone()
        .handle
        .try_typed::<Scene>()
        .expect("required preload with wrong type");
    world
        .commands()
        .entity(ctx.entity)
        .insert_if_new(SceneRoot(level_end));
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
