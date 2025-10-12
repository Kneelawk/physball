mod camera;
mod startup;
mod state;

use avian3d::PhysicsPlugins;
use bevy::app::plugin_group;
use bevy::prelude::*;
use bevy_svg::SvgPlugin;

pub fn ballphys_client_main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(SvgPlugin)
        .add_plugins(BallphysClient)
        .run()
}

plugin_group! {
    struct BallphysClient {
        startup:::BallphysStartup,
        camera:::CameraPlugin,
        state:::GameStatePlugin,
    }
}
