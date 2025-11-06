use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

pub const LEVEL_INDEX_PATH: &str = "levels/index.json";

pub fn load_level_index(cmd: &mut Commands, asset_server: &AssetServer) {
    cmd.insert_resource(LevelIndexAsset(asset_server.load(LEVEL_INDEX_PATH)));
}

pub fn on_level_index_loaded(
    mut msg: MessageReader<AssetEvent<LevelIndex>>,
    handle: Option<Res<LevelIndexAsset>>,
    asset: Res<Assets<LevelIndex>>,
) {
    if let Some(handle) = handle {
        for e in msg.read() {
            if e.is_loaded_with_dependencies(&handle.0) {
                let index = asset.get(&handle.0).expect("Level index missing");
                info!("Level index loaded: {:?}", index);
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct LevelIndexLoader;

#[derive(Debug, Clone, Resource, Deref, Reflect)]
#[reflect(Debug, Clone, Resource)]
pub struct LevelIndexAsset(pub Handle<LevelIndex>);

#[derive(Debug, Clone, Asset, Resource, Reflect)]
#[reflect(Debug, Clone, Resource)]
pub struct LevelIndex {
    pub order: Vec<String>,
    pub levels: HashMap<String, LevelRef>,
}

#[derive(Debug, Clone, Deserialize)]
struct LevelIndexJson {
    levels: Vec<LevelRef>,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
pub struct LevelRef {
    pub name: String,
    pub display: String,
    pub path: String,
}

impl AssetLoader for LevelIndexLoader {
    type Asset = LevelIndex;
    type Settings = ();
    type Error = LevelIndexLoadingError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let index: LevelIndexJson = serde_json::from_slice(&bytes)?;

        info!("Loading level index: {:?}", &index);

        let mut order = vec![];
        let mut levels = HashMap::new();
        for level in index.levels {
            order.push(level.name.clone());
            levels.insert(level.name.clone(), level);
        }

        Ok(LevelIndex { order, levels })
    }
}

#[derive(Debug, Error)]
pub enum LevelIndexLoadingError {
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error {0}")]
    Json(#[from] serde_json::Error),
}
