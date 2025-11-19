use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct MaterialLoader;

impl AssetLoader for MaterialLoader {
    type Asset = StandardMaterial;
    type Settings = ();
    type Error = MaterialLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Error)]
pub enum MaterialLoadError {}
