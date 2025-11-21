use crate::game::input::PlayerInput;
use avian3d::prelude::PhysicsGizmos;
use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gizmos)
            .add_systems(Update, toggle_gizmos);
    }
}

fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    for (_, config, _) in config_store.iter_mut() {
        config.enabled = false;
        config.depth_bias = -1.0;
    }
}

fn toggle_gizmos(
    mut input: MessageReader<PlayerInput>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    for input in input.read() {
        if let PlayerInput::ToggleGizmos = input {
            let (config, _) = config_store.config_mut::<PhysicsGizmos>();
            config.enabled ^= true;
        }
    }
}
