use crate::game::game::Player;
use crate::game::game_state::GameState;
use crate::game::levels::LevelObject;
use crate::type_expr;
use avian3d::prelude::*;
use bevy::asset::io::embedded::GetAssetServer;
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
    Collider::cuboid(0.5, 0.5, 0.5),
    CollisionEventsEnabled
)]
#[component(on_insert = finish_point_on_insert)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct FinishPoint;

fn finish_point_on_insert(mut world: DeferredWorld, ctx: HookContext) {
    let serv = world.get_asset_server();
    let mesh = serv.add(type_expr!(Mesh, Cuboid::new(0.5, 0.5, 0.5).into()));
    let material = serv.add(StandardMaterial {
        base_color: Color::linear_rgba(0.0, 0.0, 0.0, 0.25),
        emissive: LinearRgba::new(0.0, 12.0, 8.0, 0.25),
        alpha_mode: AlphaMode::Add,
        ..default()
    });
    world
        .commands()
        .entity(ctx.entity)
        .insert_if_new((Mesh3d(mesh), MeshMaterial3d(material)));
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
