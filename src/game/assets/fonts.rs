use crate::game::assets::builtin::BuiltinAssetsState;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;
use thiserror::Error;

pub const FONTS_INDEX_PATH: &str = "fonts/index.json";

pub fn load_fonts(cmd: &mut Commands, asset_server: &AssetServer) {
    cmd.insert_resource(BuiltinFontsAsset(asset_server.load(FONTS_INDEX_PATH)));
}

pub fn load_fonts_system(
    mut msg: MessageReader<AssetEvent<BuiltinFonts>>,
    mut cmd: Commands,
    mut builtin_state: ResMut<BuiltinAssetsState>,
    handle: Option<Res<BuiltinFontsAsset>>,
    asset: Res<Assets<BuiltinFonts>>,
) {
    if let Some(handle) = handle {
        for e in msg.read() {
            if e.is_loaded_with_dependencies(handle.0.id()) {
                cmd.insert_resource(asset.get(&handle.0).unwrap().clone());
                *builtin_state = BuiltinAssetsState {
                    fonts: true,
                    ..*builtin_state
                };
                info!("Builtin fonts loaded.");
                return;
            }
        }
    }
}

#[derive(Default)]
pub struct BuiltinFontsLoader;

#[derive(Debug, Clone, Resource, Reflect)]
#[reflect(Debug, Clone, Resource)]
pub struct BuiltinFontsAsset(Handle<BuiltinFonts>);

#[derive(Debug, Clone, Asset, Resource, Reflect)]
#[reflect(Debug)]
pub struct BuiltinFonts {
    pub title: Handle<Font>,
    pub text: Handle<Font>,
}

#[derive(Debug, Clone, Deserialize)]
struct BuiltinFontsIndex {
    title: String,
    text: String,
}

impl AssetLoader for BuiltinFontsLoader {
    type Asset = BuiltinFonts;
    type Settings = ();
    type Error = BuiltinFontsLoadingError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut vec = vec![];
        reader.read_to_end(&mut vec).await?;
        let index: BuiltinFontsIndex = serde_json::from_slice(&vec)?;

        Ok(BuiltinFonts {
            title: load_context.load(index.title),
            text: load_context.load(index.text),
        })
    }
}

#[derive(Debug, Error)]
pub enum BuiltinFontsLoadingError {
    #[error("IO Error {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error {0}")]
    Json(#[from] serde_json::Error),
}
