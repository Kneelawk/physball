use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "web-storage"))]
use directories::ProjectDirs;
#[cfg(not(feature = "web-storage"))]
use lazy_static::lazy_static;
#[cfg(not(feature = "web-storage"))]
use std::fs::{OpenOptions, create_dir_all};

#[cfg(not(feature = "web-storage"))]
lazy_static! {
    pub static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("com", "kneelawk", "physball")
        .expect("Unable to find user home directory");
}

pub const DEFAULT_MOUSE_SPEED: f32 = 2.5;

#[cfg(not(feature = "web-storage"))]
const PREFS_FILENAME: &str = "prefs.json";

#[cfg(feature = "web-storage")]
const PREFS_KEY: &str = "com.kneelawk.physball/prefs";

#[derive(Debug, Copy, Clone, PartialEq, Resource, Reflect, Serialize, Deserialize)]
#[reflect(Debug, Clone, PartialEq, Resource)]
pub struct GamePrefs {
    #[cfg(not(feature = "web-storage"))]
    #[serde(default = "default_window_width")]
    pub window_width: u32,
    #[cfg(not(feature = "web-storage"))]
    #[serde(default = "default_window_height")]
    pub window_height: u32,
    #[serde(default = "default_mouse_speed")]
    pub mouse_speed: f32,
}

impl Default for GamePrefs {
    fn default() -> Self {
        Self {
            #[cfg(not(feature = "web-storage"))]
            window_width: default_window_width(),
            #[cfg(not(feature = "web-storage"))]
            window_height: default_window_height(),
            mouse_speed: default_mouse_speed(),
        }
    }
}

#[cfg(not(feature = "web-storage"))]
fn default_window_width() -> u32 {
    1280
}

#[cfg(not(feature = "web-storage"))]
fn default_window_height() -> u32 {
    720
}

fn default_mouse_speed() -> f32 {
    DEFAULT_MOUSE_SPEED
}

impl GamePrefs {
    pub fn load() -> GamePrefs {
        #[cfg(not(feature = "web-storage"))]
        {
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
        #[cfg(feature = "web-storage")]
        {
            use crate::or_return;

            let window = or_return!(_r => {
                warn!("Unable to get window");
                return default();
            } : Option(web_sys::window()));
            let storage = or_return!(ret_input => {
                warn!("Unable to get storage: {ret_input:?}");
                return default()
            } : Option(Result(window.local_storage())));
            let stored = or_return!(_r => {
                return default();
            } : Option(Result(storage.get_item(PREFS_KEY))));

            info!("Loading preferences from storage");

            serde_json::from_str(&stored).unwrap_or_else(|err| {
                warn!("Error parsing preferences file {:?}", err);
                default()
            })
        }
    }

    pub fn save(&self) {
        #[cfg(not(feature = "web-storage"))]
        {
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

            match serde_json::to_writer_pretty(file, self) {
                Ok(_) => {}
                Err(err) => {
                    error!("Error writing preferences file {:?}", err);
                }
            }
        }
        #[cfg(feature = "web-storage")]
        {
            use crate::or_return;

            let window = or_return!(_r => {
                error!("Unable to get window");
                return;
            } : Option(web_sys::window()));
            let storage = or_return!(ret_input => {
                error!("Unable to get storage: {ret_input:?}");
                return;
            } : Option(Result(window.local_storage())));

            let string = or_return!(ret_input => {
                error!("Error serializing config: {ret_input:?}");
                return;
            } : Result(serde_json::to_string(self)));

            or_return!(ret_input => {
                error!("Error storing config: {ret_input:?}")
            } : Result(storage.set_item(PREFS_KEY, &string)));
        }
    }
}
