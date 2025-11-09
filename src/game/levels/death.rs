use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Default)]
pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Kill>().add_observer(on_killable_collide);
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
#[require(Sensor, CollisionEventsEnabled)]
pub struct DeathCollider;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Component, Reflect)]
#[reflect(Debug, Default, Clone, PartialEq, Hash, Component)]
pub struct Killable;

#[derive(
    Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Message, Reflect,
)]
#[reflect(Debug, Clone, PartialEq, Hash)]
pub struct Kill {
    pub to_kill: Entity,
}

impl Kill {
    pub fn new(to_kill: Entity) -> Self {
        Self { to_kill }
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Event, Reflect)]
#[reflect(Debug, Clone, PartialEq, Hash)]
pub struct PlayerDiedEvent;

fn on_killable_collide(
    e: On<CollisionStart>,
    colliders: Query<(), With<DeathCollider>>,
    killables: Query<(), With<Killable>>,
    mut kill_msg: MessageWriter<Kill>,
) {
    if colliders.contains(e.collider1) && killables.contains(e.collider2) {
        kill_msg.write(Kill::new(e.collider2));
    }
}
