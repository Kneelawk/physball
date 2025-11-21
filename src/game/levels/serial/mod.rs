pub mod error;
// pub mod kdl;
pub mod kdl_utils;
pub mod level;

use crate::game::levels::serial::error::KdlBindError;
use ::kdl::{KdlDocument, KdlError};
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use level::SerialLevel;
use std::sync::Arc;
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

        let level = match SerialLevel::bind(&doc, load_context, Arc::new(str)) {
            Ok(level) => level,
            Err(err) => {
                eprintln!("{:?}", miette::Report::new(err.clone()));
                return Err(err.into());
            }
        };

        Ok(level)
    }

    fn extensions(&self) -> &[&str] {
        &["level.kdl"]
    }
}

#[derive(Debug, Error)]
pub enum SerialLevelLoadingError {
    #[error("KDL parse error {0}")]
    KdlParse(#[from] KdlError),
    #[error("KDL bind error {0}")]
    KdlBind(#[from] KdlBindError),
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),
}
