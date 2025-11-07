pub mod kdl;
pub mod level;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use kdl::KdlLevel;
use level::SerialLevel;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct SerialLevelLoader;

impl AssetLoader for SerialLevelLoader {
    type Asset = SerialLevel;
    type Settings = ();
    type Error = SerialLevelLoadingError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut str = String::new();
        reader.read_to_string(&mut str).await?;
        let level: KdlLevel = match knus::parse(load_context.path().to_string_lossy(), &str) {
            Ok(res) => res,
            Err(err) => {
                error!("{}", miette::Report::new(err));
                return Err(SerialLevelLoadingError::LevelFormatError);
            }
        };

        Ok(level.bind(load_context))
    }
}

#[derive(Debug, Error)]
pub enum SerialLevelLoadingError {
    #[error("Level format error")]
    LevelFormatError,
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),
}
