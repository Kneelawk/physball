pub mod kdl;
pub mod level;
mod error;

use ::kdl::{KdlDocument, KdlError};
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

        let doc: KdlDocument = match str.parse::<KdlDocument>() {
            Ok(doc) => doc,
            Err(err) => {
                eprintln!("{:?}", miette::Report::new(err.clone()));
                return Err(err.into());
            }
        };

        let level = SerialLevel::from_kdl(str, doc, load_context)?;

        Ok(level)
    }
}

#[derive(Debug, Error)]
pub enum SerialLevelLoadingError {
    #[error("Level format error")]
    LevelFormatError,
    #[error("Kdl parse error {0}")]
    KdlParse(#[from] KdlError),
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),
}
