use crate::game::assets::preload::Preloads;
use crate::game::input::PlayerInput;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_screenshot_button_pressed);
    }
}

fn on_screenshot_button_pressed(
    mut input: MessageReader<PlayerInput>,
    mut cmd: Commands,
    preloads: Option<Res<Preloads>>,
) {
    for input in input.read() {
        if let PlayerInput::Screenshot = input {
            let screenshot_path = get_screenshot_path();
            cmd.spawn(Screenshot::primary_window())
                .observe(save_to_disk(screenshot_path));

            if let Some(preloads) = &preloads {
                cmd.spawn((
                    PlaybackSettings::DESPAWN.with_volume(Volume::Decibels(-10.0)),
                    AudioPlayer(preloads.camera_sound()),
                ));
            }
        }
    }
}

fn get_screenshot_path() -> PathBuf {
    let now = chrono::Local::now();
    let filename = format!("physball-screenshot-{}.png", now);

    #[cfg(not(feature = "web-storage"))]
    {
        use crate::game::dirs::USER_DIRS;

        let screenshots_dir = USER_DIRS
            .picture_dir()
            .map(|dir| dir.join("physball"))
            .unwrap_or_else(|| PathBuf::from("./"));

        if !screenshots_dir.exists() {
            if let Err(err) = std::fs::create_dir_all(&screenshots_dir) {
                warn!("Error creating screenshot dir: {:?}", err);
            }
        }

        screenshots_dir.join(filename)
    }
    #[cfg(feature = "web-storage")]
    {
        PathBuf::from("./").join(filename)
    }
}
