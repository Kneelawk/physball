mod camera;
mod game;
mod game_state;
mod gui;
mod levels;
mod menus;
mod startup;
mod state;

use avian3d::PhysicsPlugins;
use bevy::app::plugin_group;
use bevy::input_focus::InputDispatchPlugin;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::ui_widgets::UiWidgetsPlugins;
use bevy_svg::SvgPlugin;

pub fn physball_client_main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: false,
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: format!(
                        concat!(
                            "{default},",
                            "symphonia_bundle_mp3::demuxer=warn,",
                            "symphonia_format_caf::demuxer=warn,",
                            "symphonia_format_isompf4::demuxer=warn,",
                            "symphonia_format_mkv::demuxer=warn,",
                            "symphonia_format_ogg::demuxer=warn,",
                            "symphonia_format_riff::demuxer=warn,",
                            "symphonia_format_wav::demuxer=warn,",
                            "calloop::loop_logic=error,",
                            "avian3d::dynamics::solver::islands::sleeping=error,",
                        ),
                        default = bevy::log::DEFAULT_FILTER
                    ),
                    fmt_layer: |_| {
                        Some(Box::new(
                            bevy::log::tracing_subscriber::fmt::Layer::default()
                                .without_time()
                                .map_fmt_fields(
                                    bevy::log::tracing_subscriber::field::MakeExt::debug_alt,
                                )
                                .with_writer(std::io::stderr),
                        ))
                    },
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
        gui:::GuiPlugin,
        menus:::MainMenuPlugin,
        menus:::OptionsMenuPlugin,
        menus:::PauseMenuPlugin,
        levels:::LevelsPlugin,
        game_state:::GameStatePlugin,
        game:::GamePlugin,
    }
}
