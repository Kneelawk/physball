use crate::game::PROJECT_DIRS;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::{OpenOptions, create_dir_all};

pub const DEFAULT_MOUSE_SPEED: f32 = 2.5;

const PREFS_FILENAME: &str = "prefs.json";

#[derive(Debug, Copy, Clone, PartialEq, Resource, Reflect, Serialize, Deserialize)]
#[reflect(Debug, Clone, PartialEq, Resource)]
pub struct GamePrefs {
    #[serde(default = "default_window_width")]
    pub window_width: u32,
    #[serde(default = "default_window_height")]
    pub window_height: u32,
    #[serde(default = "default_mouse_speed")]
    pub mouse_speed: f32,
}

impl Default for GamePrefs {
    fn default() -> Self {
        Self {
            window_width: default_window_width(),
            window_height: default_window_height(),
            mouse_speed: default_mouse_speed(),
        }
    }
}

fn default_window_width() -> u32 {
    1280
}

fn default_window_height() -> u32 {
    720
}

fn default_mouse_speed() -> f32 {
    DEFAULT_MOUSE_SPEED
}

impl GamePrefs {
    pub fn load() -> GamePrefs {
        let path = PROJECT_DIRS.preference_dir().join(PREFS_FILENAME);

        info!("Loading preferences from {:?}", &path);

        if !path.exists() {
            info!("Preferences file does not exist. Using defaults...");
            return default();
        }

        let file = match OpenOptions::new().read(true).open(path) {
            Ok(f) => f,
            Err(err) => {
                warn!("Error opening preferences file {:?}", err);
                return default();
            }
        };

        serde_json::from_reader(file).unwrap_or_else(|err| {
            warn!("Error parsing preferences file {:?}", err);
            default()
        })
    }

    pub fn save(&self) {
        let path = PROJECT_DIRS.preference_dir().join(PREFS_FILENAME);

        info!("Writing preferences to {:?}", &path);

        if !path.parent().unwrap().exists() {
            match create_dir_all(path.parent().unwrap()) {
                Ok(_) => {}
                Err(err) => {
                    error!("Error creating preferences dir {:?}", err);
                    return;
                }
            }
        }

        let file = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
        {
            Ok(f) => f,
            Err(err) => {
                error!("Error opening preferences file for writing {:?}", err);
                return;
            }
        };

        match serde_json::to_writer(file, self) {
            Ok(_) => {}
            Err(err) => {
                error!("Error writing preferences file {:?}", err);
            }
        }
    }
}
