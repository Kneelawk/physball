mod camera;
mod gui;
mod main_menu;
mod startup;
mod state;

use avian3d::PhysicsPlugins;
use bevy::app::plugin_group;
use bevy::input_focus::InputDispatchPlugin;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::prelude::*;
use bevy::ui_widgets::UiWidgetsPlugins;
use bevy_svg::SvgPlugin;

pub fn ballphys_client_main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            SvgPlugin,
            BallphysClient,
            UiWidgetsPlugins,
            InputDispatchPlugin,
            TabNavigationPlugin,
        ))
        .run()
}

plugin_group! {
    struct BallphysClient {
        state:::GameStatePlugin,
        startup:::BallphysStartup,
        camera:::CameraPlugin,
        main_menu:::MainMenuPlugin,
    }
}
