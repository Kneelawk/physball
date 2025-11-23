mod assets;
mod camera;
mod game_state;
mod gizmos;
mod gui;
mod input;
mod levels;
mod logic;
mod menus;
mod music;
mod settings;
mod startup;
mod state;

use crate::game::settings::GamePrefs;
use avian3d::PhysicsPlugins;
use avian3d::prelude::PhysicsDebugPlugin;
use bevy::app::plugin_group;
use bevy::input_focus::InputDispatchPlugin;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::ui_widgets::UiWidgetsPlugins;
use bevy::window::EnabledButtons;
use bevy_rich_text3d::Text3dPlugin;
use bevy_svg::SvgPlugin;

pub const CANVAS_ID: &str = "game";

pub fn physball_client_main() -> AppExit {
    let prefs = GamePrefs::load();

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        #[cfg(feature = "window-resize")]
                        resizable: false,
                        #[cfg(feature = "window-resize")]
                        resolution: (prefs.window_width, prefs.window_height).into(),
                        enabled_buttons: EnabledButtons {
                            minimize: true,
                            maximize: false,
                            close: true,
                        },
                        canvas: Some(format!("#{CANVAS_ID}")),
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
                            "bevy_asset::server::info=error,",
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
            Text3dPlugin::default(),
            BallphysClient,
            UiWidgetsPlugins,
            InputDispatchPlugin,
            TabNavigationPlugin,
            PhysicsDebugPlugin,
        ))
        .insert_resource(prefs)
        .run()
}

plugin_group! {
    struct BallphysClient {
        state:::GameStatePlugin,
        assets:::BuiltinAssetsPlugin,
        startup:::BallphysStartup,
        camera:::CameraPlugin,
        input:::InputPlugin,
        gui:::GuiPlugin,
        music:::MusicPlugin,
        menus:::MainMenuPlugin,
        menus:::OptionsMenuPlugin,
        menus:::LoadingScreenPlugin,
        menus:::PauseMenuPlugin,
        menus:::FinishMenuPlugin,
        levels:::LevelsPlugin,
        levels::finish_point:::FinishPointPlugin,
        levels::button:::ButtonPlugin,
        levels::death:::DeathPlugin,
        game_state:::GameStatePlugin,
        logic:::GamePlugin,
        gizmos:::GizmosPlugin,
    }
}
