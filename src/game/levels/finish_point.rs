use bevy::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Event, Reflect)]
pub struct LevelFinishEvent;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[require(Transform)]
#[component(on_insert = finish_point_on_insert)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct FinishPoint;

fn finish_point_on_insert(mut world: DeferredWorld, ctx: HookContext) {

}
